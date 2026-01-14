//! Application bootstrapping
//!
//! This module handles the wiring of all application dependencies.
//! Infrastructure implementations are instantiated here and injected
//! into application layer use cases.

use std::sync::Arc;

use crate::app::auth::{LoginUseCase, RegisterUseCase};
use crate::app::ports::TokenService;
use crate::app::user::{CreateUserUseCase, GetUserUseCase, ListUsersUseCase, UpdateUserUseCase};
use crate::domain::user::UserRepository;
use crate::infra::auth::JwtTokenService;
use crate::infra::config::{self, Config};
use crate::infra::persistence::SeaOrmUserRepository;
use crate::presentation::AppState;

/// Bootstrap error type
#[derive(Debug)]
pub struct BootstrapError(pub String);

impl std::fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bootstrap error: {}", self.0)
    }
}

impl std::error::Error for BootstrapError {}

/// Bootstrap the application and return the configured AppState
///
/// This function:
/// 1. Connects to the database
/// 2. Creates infrastructure implementations (repositories, services)
/// 3. Injects them into application use cases
/// 4. Returns a fully configured AppState
pub async fn create_app_state(config: Config) -> Result<AppState, BootstrapError> {
    // Infrastructure layer: Database connection (internal to bootstrap)
    let db = Arc::new(
        config::database::connect()
            .await
            .map_err(|e| BootstrapError(format!("Failed to connect to database: {}", e)))?,
    );

    // Infrastructure layer: Create repository implementation
    let user_repository: Arc<dyn UserRepository> = Arc::new(SeaOrmUserRepository::new(db));

    // Infrastructure layer: Create token service
    let token_service: Arc<dyn TokenService> = Arc::new(JwtTokenService::new());

    // Application layer: Create use cases
    let login_use_case = Arc::new(LoginUseCase::new(
        user_repository.clone(),
        token_service.clone(),
    ));
    let register_use_case = Arc::new(RegisterUseCase::new(user_repository.clone()));
    let create_user_use_case = Arc::new(CreateUserUseCase::new(user_repository.clone()));
    let get_user_use_case = Arc::new(GetUserUseCase::new(user_repository.clone()));
    let list_users_use_case = Arc::new(ListUsersUseCase::new(user_repository.clone()));
    let update_user_use_case = Arc::new(UpdateUserUseCase::new(user_repository.clone()));

    Ok(AppState {
        config,
        login_use_case,
        register_use_case,
        create_user_use_case,
        get_user_use_case,
        list_users_use_case,
        update_user_use_case,
    })
}
