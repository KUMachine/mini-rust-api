use axum::{Json, Router, extract::State, routing::post};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use validator::Validate;

use crate::AppState;
use crate::entity::users::{self, Entity as Users};
use crate::errors::{AppError, AppResult};
use crate::models::user::UserResponse;
use crate::response::ApiResponse;
use crate::validators::user::{LoginRequest, RegisterRequest};

use super::service::{create_jwt_token, hash_password, verify_password};
use super::types::{AuthBody, AuthError};

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<AuthBody>),
        (status = 401, description = "Invalid credentials"),
        (status = 400, description = "Invalid input")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<AuthBody>>> {
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(vec![e.to_string()]))?;

    // Find user by email
    let user = Users::find()
        .filter(users::Column::Email.eq(&payload.email))
        .one(state.db.as_ref())
        .await?
        .ok_or(AuthError::WrongCredentials)?;

    // Verify password
    if !verify_password(&payload.password, &user.password_hash)? {
        return Err(AuthError::WrongCredentials.into());
    }

    // Create JWT token
    let token = create_jwt_token(user.email.clone(), user.id)?;

    Ok(Json(ApiResponse::ok(AuthBody::new(token))))
}

/// Register a new user account
#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered successfully", body = ApiResponse<UserResponse>),
        (status = 422, description = "Validation error"),
        (status = 400, description = "User already exists")
    ),
    tag = "auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(vec![e.to_string()]))?;

    // Check if user already exists
    if Users::find()
        .filter(users::Column::Email.eq(&payload.email))
        .one(state.db.as_ref())
        .await?
        .is_some()
    {
        return Err(AppError::ValidationError(vec![
            "User with this email already exists".to_string(),
        ]));
    }

    // Hash password
    let password_hash = hash_password(&payload.password)?;

    // Create new user
    let new_user = users::ActiveModel {
        first_name: Set(payload.first_name),
        last_name: Set(payload.last_name),
        age: Set(payload.age),
        email: Set(payload.email),
        password_hash: Set(password_hash),
        create_at: Set(Utc::now().naive_utc().date()),
        ..Default::default()
    };

    let user = new_user.insert(state.db.as_ref()).await?;
    Ok(Json(ApiResponse::ok(UserResponse::from(user))))
}

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}
