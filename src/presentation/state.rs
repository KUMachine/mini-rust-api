//! Application state
//!
//! Contains the shared application state passed to all handlers.

use crate::app::auth::{LoginUseCase, RegisterUseCase};
use crate::app::user::{CreateUserUseCase, GetUserUseCase, ListUsersUseCase, UpdateUserUseCase};
use crate::infra::Config;
use sea_orm::DbConn;
use std::sync::Arc;

/// Shared application state
///
/// Contains all dependencies that need to be available to request handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: Arc<DbConn>,
    // Auth use cases
    pub login_use_case: Arc<LoginUseCase>,
    pub register_use_case: Arc<RegisterUseCase>,
    // User use cases
    pub create_user_use_case: Arc<CreateUserUseCase>,
    pub get_user_use_case: Arc<GetUserUseCase>,
    pub list_users_use_case: Arc<ListUsersUseCase>,
    pub update_user_use_case: Arc<UpdateUserUseCase>,
}
