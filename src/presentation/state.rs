//! Application state
//!
//! Contains the shared application state passed to all handlers.
//! Handlers interact with use cases only, which abstract away persistence.
//! The user_repository is exposed for role lookups in the auth middleware.

use crate::app::auth::{LoginUseCase, RegisterUseCase};
use crate::app::user::{CreateUserUseCase, GetUserUseCase, ListUsersUseCase, UpdateUserUseCase};
use crate::domain::user::UserRepository;
use crate::infra::Config;
use std::sync::Arc;

/// Shared application state
///
/// Contains all dependencies that need to be available to request handlers.
/// Exposes use cases and the user repository (a domain trait, not infrastructure).
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    // Repository (domain trait) - used by auth middleware for role lookups
    pub user_repository: Arc<dyn UserRepository>,
    // Auth use cases
    pub login_use_case: Arc<LoginUseCase>,
    pub register_use_case: Arc<RegisterUseCase>,
    // User use cases
    pub create_user_use_case: Arc<CreateUserUseCase>,
    pub get_user_use_case: Arc<GetUserUseCase>,
    pub list_users_use_case: Arc<ListUsersUseCase>,
    pub update_user_use_case: Arc<UpdateUserUseCase>,
}
