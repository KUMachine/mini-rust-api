use crate::errors::{AppError, AppResult};
use crate::models::user::UserResponse;
use crate::response::ApiResponse;
use chrono::Utc;

use crate::AppState;
use crate::entity::users;
use crate::entity::users::Entity as Users;
use crate::validators::user::CreateUserRequest;
use axum::{Json, Router, extract::State, routing::get};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use validator::Validate;

pub fn routes() -> Router<AppState> {
    Router::new().route("/users", get(list_users).post(create_user))
}

/// List all users
#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List of users", body = ApiResponse<Vec<UserResponse>>)
    ),
    tag = "users"
)]
pub async fn list_users(
    State(state): State<AppState>,
) -> AppResult<Json<ApiResponse<Vec<UserResponse>>>> {
    let users = Users::find().all(&*state.db).await?;

    Ok(Json(ApiResponse::ok(
        users.into_iter().map(UserResponse::from).collect(),
    )))
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Invalid input")
    ),
    tag = "users"
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    let new_user = users::ActiveModel {
        first_name: Set(payload.first_name),
        last_name: Set(payload.last_name),
        age: Set(payload.age),
        create_at: Set(Utc::now().naive_utc().date()),
        ..Default::default()
    };

    let inserted = new_user.insert(&*state.db).await.unwrap();
    Ok(Json(ApiResponse::ok(UserResponse::from(inserted))))
}
