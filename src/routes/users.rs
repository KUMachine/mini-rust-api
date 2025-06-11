use crate::models::user::{CreateUser, UserResponse};
use crate::response::{ ApiResponse, ApiErrorResponse};
use crate::errors::{AppError, AppResult};

use axum::{
    extract::State,
    routing::{get},
    Json, Router,
};
use sea_orm::{EntityTrait, ActiveModelTrait, Set};

use crate::entity::users;
use crate::entity::users::Entity as Users;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
}

async fn list_users(State(state): State<AppState>) -> AppResult<Json<ApiResponse<Vec<UserResponse>>>> {
    let users = Users::find()
        .all(&*state.db)
        .await?;

    Ok(Json(ApiResponse::ok(users.into_iter().map(UserResponse::from).collect())))
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    let new_user = users::ActiveModel {
        first_name: Set(payload.first_name),
        last_name: Set(payload.last_name),
        age: Set(payload.age),
        ..Default::default()
    };

    let inserted = new_user.insert(&*state.db).await.unwrap();
    Ok(Json(ApiResponse::ok(UserResponse::from(inserted))))
}