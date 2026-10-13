//! Application state
//!
//! Contains the shared application state passed to all handlers.
//! Handlers interact with use cases only, which abstract away persistence
//! and other infrastructure.

use crate::app::auth::{AuthenticateUseCase, LoginUseCase, RegisterUseCase};
use crate::app::user::{CreateUserUseCase, GetUserUseCase, ListUsersUseCase, UpdateUserUseCase};
use std::sync::Arc;

/// Shared application state
///
/// Contains all use cases that need to be available to request handlers.
#[derive(Clone)]
pub struct AppState {
    // Auth use cases
    pub authenticate_use_case: Arc<AuthenticateUseCase>,
    pub login_use_case: Arc<LoginUseCase>,
    pub register_use_case: Arc<RegisterUseCase>,
    // User use cases
    pub create_user_use_case: Arc<CreateUserUseCase>,
    pub get_user_use_case: Arc<GetUserUseCase>,
    pub list_users_use_case: Arc<ListUsersUseCase>,
    pub update_user_use_case: Arc<UpdateUserUseCase>,
}
