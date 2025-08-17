use crate::auth::hash_password;
use crate::errors::{AppError, AppResult};
use crate::extractors::{ValidatedJson, ValidatedPagination};
use crate::models::user::UserResponse;
use crate::pagination::PaginationRequest;
use crate::response::{ApiErrorResponse, ApiResponse};
use chrono::Utc;

use crate::AppState;
use crate::entity::users;
use crate::entity::users::Entity as Users;
use crate::validators::user::CreateUserRequest;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, QueryOrder, QuerySelect, Set};
use validator::Validate;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", get(get_user))
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
    ValidatedPagination(pagination): ValidatedPagination<PaginationRequest>,
) -> AppResult<Json<ApiResponse<Vec<UserResponse>>>> {
    let page = pagination.page;
    let rows_per_page = pagination.rows_per_page;
    let offset = ((page - 1) * rows_per_page) as u64;

    let users = Users::find()
        .order_by_desc(users::Column::Id)
        .offset(offset)
        .limit(rows_per_page as u64)
        .all(state.db.as_ref())
        .await?;

    let total = Users::find().count(state.db.as_ref()).await?;

    Ok(Json(ApiResponse::with_pagination(
        users.into_iter().map(UserResponse::from).collect(),
        total,
        rows_per_page,
        page,
    )))
}

/// Get a user by ID
#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = ApiResponse<UserResponse>),
        (status = 404, description = "User not found"),
        (status = 401, description = "Unauthorized - Valid JWT token required")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    let user = Users::find_by_id(id)
        .one(state.db.as_ref())
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ApiResponse::ok(UserResponse::from(user))))
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created successfully", body = ApiResponse<UserResponse>),
        (status = 422, description = "Validation error", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized - Valid JWT token required")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn create_user(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateUserRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    payload.validate().map_err(|e| {
        let error_messages: Vec<String> = e
            .field_errors()
            .iter()
            .flat_map(|(_, errors)| {
                errors.iter().map(|error| {
                    error
                        .message
                        .as_ref()
                        .map(|msg| msg.to_string())
                        .unwrap_or_else(|| "Invalid value".to_string())
                })
            })
            .collect();
        AppError::ValidationError(error_messages)
    })?;

    // Hash password
    let password_hash = hash_password(&payload.password)?;

    let new_user = users::ActiveModel {
        first_name: Set(payload.first_name),
        last_name: Set(payload.last_name),
        age: Set(payload.age),
        email: Set(payload.email),
        password_hash: Set(password_hash),
        create_at: Set(Utc::now().naive_utc().date()),
        ..Default::default()
    };

    let inserted = new_user.insert(state.db.as_ref()).await?;
    Ok(Json(ApiResponse::ok(UserResponse::from(inserted))))
}
