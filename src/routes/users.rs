use crate::models::user::{UserResponse};
use crate::response::{ ApiResponse};
use crate::errors::{AppError, AppResult};

use axum::{
    extract::State,
    routing::{get},
    Json, Router,
};
use sea_orm::{EntityTrait, ActiveModelTrait, Set};
use validator::Validate;
use crate::entity::users;
use crate::entity::users::Entity as Users;
use crate::AppState;
use crate::validators::user::CreateUserRequest;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
}

pub async fn list_users(State(state): State<AppState>) -> AppResult<Json<ApiResponse<Vec<UserResponse>>>> {
    let users = Users::find()
        .all(&*state.db)
        .await?;

    Ok(Json(ApiResponse::ok(users.into_iter().map(UserResponse::from).collect())))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    payload.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;
    let new_user = users::ActiveModel {
        first_name: Set(payload.first_name),
        last_name: Set(payload.last_name),
        age: Set(payload.age),
        ..Default::default()
    };

    let inserted = new_user.insert(&*state.db).await.unwrap();
    Ok(Json(ApiResponse::ok(UserResponse::from(inserted))))
}