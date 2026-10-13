//! Authentication API handlers
//!
//! Handles login and registration endpoints.

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};

use crate::app::ApplicationError;
use crate::app::auth::{AuthToken, LoginCommand, RegisterCommand};
use crate::app::user::UserResponse;
use crate::presentation::extractors::JsonBody;
use crate::presentation::responses::{ApiErrorResponse, ApiResponse};
use crate::presentation::state::AppState;

/// Login endpoint
#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginCommand,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<AuthToken>),
        (status = 400, description = "Malformed email or request body", body = ApiErrorResponse),
        (status = 401, description = "Invalid credentials", body = ApiErrorResponse),
        (status = 422, description = "Request body does not match the schema", body = ApiErrorResponse)
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    JsonBody(command): JsonBody<LoginCommand>,
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
        (status = 201, description = "User registered successfully", body = ApiResponse<UserResponse>),
        (status = 400, description = "Domain validation failed (email, password, profile)", body = ApiErrorResponse),
        (status = 409, description = "A user with this email already exists", body = ApiErrorResponse),
        (status = 422, description = "Request body does not match the schema", body = ApiErrorResponse)
    ),
    tag = "auth"
)]
pub async fn register(
    State(state): State<AppState>,
    JsonBody(command): JsonBody<RegisterCommand>,
) -> Result<(StatusCode, Json<ApiResponse<UserResponse>>), ApplicationError> {
    let user = state.register_use_case.execute(command).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(user))))
}

/// Create authentication routes
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}
