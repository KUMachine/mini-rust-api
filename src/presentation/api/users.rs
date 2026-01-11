//! User management API handlers
//!
//! CRUD operations for user management.

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::app::ApplicationError;
use crate::app::user::{CreateUserCommand, ListUsersQuery, UpdateUserCommand, UserResponse};
use crate::presentation::extractors::{ValidatedJson, ValidatedPagination};
use crate::presentation::responses::{ApiErrorResponse, ApiResponse, PaginationRequest};
use crate::presentation::state::AppState;

/// Create user routes
pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", get(get_user).put(update_user))
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
) -> Result<Json<ApiResponse<Vec<UserResponse>>>, ApplicationError> {
    let page = pagination.page;
    let rows_per_page = pagination.rows_per_page;

    let query = ListUsersQuery {
        page: page as u64,
        rows_per_page: rows_per_page as u64,
    };

    let (users, total) = state.list_users_use_case.execute(query).await?;

    Ok(Json(ApiResponse::with_pagination(
        users,
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
) -> Result<Json<ApiResponse<UserResponse>>, ApplicationError> {
    let user = state.get_user_use_case.execute(id).await?;
    Ok(Json(ApiResponse::ok(user)))
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserCommand,
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
    ValidatedJson(command): ValidatedJson<CreateUserCommand>,
) -> Result<Json<ApiResponse<UserResponse>>, ApplicationError> {
    let user = state.create_user_use_case.execute(command).await?;
    Ok(Json(ApiResponse::ok(user)))
}

/// Update a user
#[utoipa::path(
    put,
    path = "/users/{id}",
    request_body = UpdateUserCommand,
    responses(
        (status = 200, description = "User updated successfully", body = ApiResponse<UserResponse>),
        (status = 422, description = "Validation error", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized - Valid JWT token required")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    ValidatedJson(command): ValidatedJson<UpdateUserCommand>,
) -> Result<Json<ApiResponse<UserResponse>>, ApplicationError> {
    let user = state.update_user_use_case.execute(id, command).await?;
    Ok(Json(ApiResponse::ok(user)))
}
