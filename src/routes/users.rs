use crate::errors::{AppError, AppResult};
use crate::models::user::UserResponse;
use crate::pagination::PaginationRequest;
use crate::response::ApiResponse;
use bcrypt::{DEFAULT_COST, hash};
use chrono::Utc;

use crate::AppState;
use crate::entity::users;
use crate::entity::users::Entity as Users;
use crate::validators::user::CreateUserRequest;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, QueryOrder, QuerySelect, Set};
use validator::Validate;

pub fn routes() -> Router<AppState> {
    Router::new().route("/users", get(list_users).post(create_user))
}

/// List all users
#[utoipa::path(
    get,
    path = "/users",
    params(
        ("page" = Option<u32>, Query, description = "Page number (default: 1)"),
        ("rowsPerPage" = Option<u32>, Query, description = "Number of items per page (default: 10)")
    ),
    responses(
        (status = 200, description = "List of users", body = ApiResponse<Vec<UserResponse>>),
        (status = 401, description = "Unauthorized - Valid JWT token required")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn list_users(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationRequest>,
) -> AppResult<Json<ApiResponse<Vec<UserResponse>>>> {
    let page = pagination.page;
    let rows_per_page = pagination.rows_per_page;
    let offset = ((page - 1) * rows_per_page) as u64;

    let users = Users::find()
        .order_by_desc(users::Column::Id)
        .offset(offset)
        .limit(rows_per_page as u64)
        .all(&*state.db)
        .await?;

    let total = Users::find().count(&*state.db).await?;

    Ok(Json(ApiResponse::with_pagination(
        users.into_iter().map(UserResponse::from).collect(),
        total,
        rows_per_page,
        page,
    )))
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created successfully", body = ApiResponse<UserResponse>),
        (status = 422, description = "Validation error"),
        (status = 401, description = "Unauthorized - Valid JWT token required")
    ),
    security(
        ("bearer_auth" = [])
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

    // Hash password
    let password_hash = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| AppError::Unexpected("Failed to hash password".to_string()))?;

    let new_user = users::ActiveModel {
        first_name: Set(payload.first_name),
        last_name: Set(payload.last_name),
        age: Set(payload.age),
        email: Set(payload.email),
        password_hash: Set(password_hash),
        create_at: Set(Utc::now().naive_utc().date()),
        ..Default::default()
    };

    let inserted = new_user.insert(&*state.db).await?;
    Ok(Json(ApiResponse::ok(UserResponse::from(inserted))))
}
