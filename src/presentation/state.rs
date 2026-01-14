//! Application state
//!
//! Contains the shared application state passed to all handlers.
//! Note: Infrastructure details like database connections are NOT exposed here.
//! Handlers interact with use cases only, which abstract away persistence.

use crate::app::auth::{LoginUseCase, RegisterUseCase};
use crate::app::user::{CreateUserUseCase, GetUserUseCase, ListUsersUseCase, UpdateUserUseCase};
use crate::infra::Config;
use std::sync::Arc;

/// Shared application state
///
/// Contains all dependencies that need to be available to request handlers.
/// Only exposes use cases - no infrastructure details leak through.
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    // Auth use cases
    pub login_use_case: Arc<LoginUseCase>,
    pub register_use_case: Arc<RegisterUseCase>,
    // User use cases
    pub create_user_use_case: Arc<CreateUserUseCase>,
    pub get_user_use_case: Arc<GetUserUseCase>,
    pub list_users_use_case: Arc<ListUsersUseCase>,
    pub update_user_use_case: Arc<UpdateUserUseCase>,
}
