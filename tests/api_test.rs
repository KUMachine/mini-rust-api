//! HTTP-level tests for the API
//!
//! These tests drive the real router with an in-memory repository, covering
//! routing, extractors, authentication, authorization and error formatting.

// Test helpers fail loudly by design
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use mini_rust_api::bootstrap::build_app_state;
use mini_rust_api::domain::user::{Email, Role, User, UserRepository};
use mini_rust_api::infra::auth::JwtTokenService;
use mini_rust_api::infra::persistence::InMemoryUserRepository;
use mini_rust_api::presentation::create_router;
use serde_json::{Value, json};
use tower::ServiceExt;

const PASSWORD: &str = "SecurePass123";

struct TestApp {
    router: Router,
    repository: Arc<InMemoryUserRepository>,
}

impl TestApp {
    fn new() -> Self {
        let repository = Arc::new(InMemoryUserRepository::new());
        let state = build_app_state(
            repository.clone(),
            Arc::new(JwtTokenService::new("test-secret")),
        );
        Self {
            router: create_router(state),
            repository,
        }
    }

    async fn request(
        &self,
        method: Method,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let request = match body {
            Some(body) => builder
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string())),
            None => builder.body(Body::empty()),
        }
        .unwrap();

        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
        (status, body)
    }

    async fn register(&self, email: &str) -> (StatusCode, Value) {
        self.request(
            Method::POST,
            "/register",
            None,
            Some(json!({
                "email": email,
                "password": PASSWORD,
                "first_name": "Test",
                "last_name": "User",
                "age": 30
            })),
        )
        .await
    }

    async fn login(&self, email: &str) -> String {
        let (status, body) = self
            .request(
                Method::POST,
                "/login",
                None,
                Some(json!({ "email": email, "password": PASSWORD })),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "login failed: {body}");
        body["data"]["access_token"].as_str().unwrap().to_string()
    }

    /// Register a regular user and return (id, token)
    async fn user(&self, email: &str) -> (i64, String) {
        let (status, body) = self.register(email).await;
        assert_eq!(status, StatusCode::CREATED, "register failed: {body}");
        (
            body["data"]["id"].as_i64().unwrap(),
            self.login(email).await,
        )
    }

    /// Seed an admin directly through the repository and return its token
    async fn admin(&self) -> String {
        let email = "admin@example.com";
        let mut admin = User::register(
            Email::try_from(email).unwrap(),
            PASSWORD.to_string(),
            "Admin".to_string(),
            "User".to_string(),
            40,
        )
        .unwrap();
        admin.add_role(Role::Admin);
        self.repository.save(&mut admin).await.unwrap();
        self.login(email).await
    }
}

fn error_code(body: &Value) -> &str {
    body["errors"][0]["code"].as_str().unwrap_or_default()
}

#[tokio::test]
async fn health_check_responds() {
    let app = TestApp::new();
    let (status, _) = app.request(Method::GET, "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn openapi_document_is_served() {
    let app = TestApp::new();
    let (status, body) = app
        .request(Method::GET, "/api-docs/openapi.json", None, None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["paths"]["/register"]["post"]["responses"]["201"].is_object());
    assert!(body["components"]["securitySchemes"]["bearer_auth"].is_object());
}

#[tokio::test]
async fn register_creates_user_with_default_role() {
    let app = TestApp::new();
    let (status, body) = app.register("  New.User@Example.com ").await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["email"], "new.user@example.com");
    assert_eq!(body["data"]["roles"], json!(["user"]));
}

#[tokio::test]
async fn register_rejects_duplicate_email() {
    let app = TestApp::new();
    app.register("dup@example.com").await;
    let (status, body) = app.register("DUP@example.com").await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&body), "EMAIL_ALREADY_EXISTS");
}

#[tokio::test]
async fn register_rejects_weak_password() {
    let app = TestApp::new();
    let (status, body) = app
        .request(
            Method::POST,
            "/register",
            None,
            Some(json!({
                "email": "weak@example.com",
                "password": "alllowercase",
                "first_name": "Test",
                "last_name": "User",
                "age": 30
            })),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&body), "PASSWORD_TOO_WEAK");
}

#[tokio::test]
async fn malformed_json_is_reported_as_json_api_error() {
    let app = TestApp::new();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/register")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{not json"))
        .unwrap();
    let response = app.router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let (status, body) = app
        .request(Method::POST, "/login", None, Some(json!({ "email": 1 })))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error_code(&body), "JSON_DATA_ERROR");
}

#[tokio::test]
async fn login_rejects_wrong_password_and_unknown_email_alike() {
    let app = TestApp::new();
    app.register("known@example.com").await;

    for (email, password) in [
        ("known@example.com", "WrongPass123"),
        ("unknown@example.com", PASSWORD),
    ] {
        let (status, body) = app
            .request(
                Method::POST,
                "/login",
                None,
                Some(json!({ "email": email, "password": password })),
            )
            .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(error_code(&body), "INVALID_CREDENTIALS");
    }
}

#[tokio::test]
async fn protected_routes_require_valid_token() {
    let app = TestApp::new();

    let (status, body) = app.request(Method::GET, "/users/1", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(&body), "UNAUTHORIZED");

    let (status, _) = app
        .request(Method::GET, "/users/1", Some("not-a-jwt"), None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn users_can_only_view_their_own_profile() {
    let app = TestApp::new();
    let (alice_id, alice_token) = app.user("alice@example.com").await;
    let (bob_id, _) = app.user("bob@example.com").await;

    let (status, body) = app
        .request(
            Method::GET,
            &format!("/users/{alice_id}"),
            Some(&alice_token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["email"], "alice@example.com");

    let (status, body) = app
        .request(
            Method::GET,
            &format!("/users/{bob_id}"),
            Some(&alice_token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_code(&body), "FORBIDDEN");
}

#[tokio::test]
async fn only_admins_can_list_users() {
    let app = TestApp::new();
    let (_, user_token) = app.user("user@example.com").await;
    let admin_token = app.admin().await;

    let (status, _) = app
        .request(Method::GET, "/users", Some(&user_token), None)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, body) = app
        .request(
            Method::GET,
            "/users?page=1&rowsPerPage=1",
            Some(&admin_token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["meta"],
        json!({ "count": 2, "rowsPerPage": 1, "page": 1 })
    );
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    // Newest first: the admin, with roles in a stable order
    assert_eq!(body["data"][0]["roles"], json!(["admin", "user"]));
}

#[tokio::test]
async fn list_users_validates_pagination() {
    let app = TestApp::new();
    let admin_token = app.admin().await;

    let (status, body) = app
        .request(
            Method::GET,
            "/users?rowsPerPage=1000",
            Some(&admin_token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&body), "PAGINATION_VALIDATION_ERROR");
}

#[tokio::test]
async fn only_admins_can_create_users() {
    let app = TestApp::new();
    let (_, user_token) = app.user("user@example.com").await;
    let admin_token = app.admin().await;
    let new_user = json!({
        "email": "created@example.com",
        "password": PASSWORD,
        "first_name": "Created",
        "last_name": "User",
        "age": 25
    });

    let (status, _) = app
        .request(
            Method::POST,
            "/users",
            Some(&user_token),
            Some(new_user.clone()),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, body) = app
        .request(Method::POST, "/users", Some(&admin_token), Some(new_user))
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["email"], "created@example.com");
}

#[tokio::test]
async fn update_rejects_email_owned_by_another_user() {
    let app = TestApp::new();
    let (alice_id, alice_token) = app.user("alice@example.com").await;
    app.user("bob@example.com").await;

    let update = |email: &str| json!({ "email": email, "first_name": "Alice", "last_name": "Updated", "age": 31 });

    let (status, body) = app
        .request(
            Method::PUT,
            &format!("/users/{alice_id}"),
            Some(&alice_token),
            Some(update("bob@example.com")),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&body), "EMAIL_ALREADY_EXISTS");

    let (status, body) = app
        .request(
            Method::PUT,
            &format!("/users/{alice_id}"),
            Some(&alice_token),
            Some(update("alice@example.com")),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["last_name"], "Updated");
    assert_eq!(body["data"]["age"], 31);
}

#[tokio::test]
async fn admin_gets_not_found_for_missing_user() {
    let app = TestApp::new();
    let admin_token = app.admin().await;

    let (status, body) = app
        .request(Method::GET, "/users/999", Some(&admin_token), None)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&body), "USER_NOT_FOUND");
}
