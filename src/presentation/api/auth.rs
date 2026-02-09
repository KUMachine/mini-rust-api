//! Authentication API handlers
//!
//! Handles login and registration endpoints.

use axum::{Json, Router, extract::State, routing::post};

use crate::app::ApplicationError;
use crate::app::auth::{AuthToken, LoginCommand, RegisterCommand};
use crate::app::user::UserResponse;
use crate::presentation::responses::ApiResponse;
use crate::presentation::state::AppState;

/// Login endpoint
#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginCommand,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<AuthToken>),
        (status = 401, description = "Invalid credentials"),
        (status = 422, description = "Validation error")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(command): Json<LoginCommand>,
) -> Result<Json<ApiResponse<AuthToken>>, ApplicationError> {
    let auth_token = state.login_use_case.execute(command).await?;
    Ok(Json(ApiResponse::ok(auth_token)))
}

/// Register a new user account
#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterCommand,
    responses(
        (status = 200, description = "User registered successfully", body = ApiResponse<UserResponse>),
        (status = 422, description = "Validation error"),
        (status = 400, description = "User already exists")
    ),
    tag = "auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(command): Json<RegisterCommand>,
) -> Result<Json<ApiResponse<UserResponse>>, ApplicationError> {
    let user = state.register_use_case.execute(command).await?;
    Ok(Json(ApiResponse::ok(user)))
}

/// Create authentication routes
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}
