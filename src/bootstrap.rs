//! Application bootstrapping
//!
//! This module handles the wiring of all application dependencies.
//! Infrastructure implementations are instantiated here and injected
//! into application layer use cases.

use std::sync::Arc;

use crate::app::auth::{AuthenticateUseCase, LoginUseCase, RegisterUseCase};
use crate::app::ports::TokenService;
use crate::app::user::{CreateUserUseCase, GetUserUseCase, ListUsersUseCase, UpdateUserUseCase};
use crate::domain::user::UserRepository;
use crate::infra::auth::JwtTokenService;
use crate::infra::config::{self, Config, Storage};
use crate::infra::persistence::{InMemoryUserRepository, SeaOrmUserRepository};
use crate::presentation::AppState;
use thiserror::Error;

/// Bootstrap error type
#[derive(Debug, Error)]
pub enum BootstrapError {
    #[error("failed to connect to database")]
    Database(#[source] sea_orm::DbErr),
}

/// Bootstrap the application and return the configured AppState
///
/// This function:
/// 1. Connects to the configured storage
/// 2. Creates infrastructure implementations (repositories, services)
/// 3. Injects them into application use cases
pub async fn create_app_state(config: &Config) -> Result<AppState, BootstrapError> {
    // Infrastructure layer: Create adapters
    let user_repository: Arc<dyn UserRepository> = match &config.storage {
        Storage::Postgres(database) => {
            let db = config::database::connect(database)
                .await
                .map_err(BootstrapError::Database)?;
            Arc::new(SeaOrmUserRepository::new(Arc::new(db)))
        }
        Storage::Memory => {
            tracing::warn!("STORAGE=memory: data lives in process memory and is lost on restart");
            Arc::new(InMemoryUserRepository::new())
        }
    };
    let token_service: Arc<dyn TokenService> =
        Arc::new(JwtTokenService::new(config.auth.jwt_secret.as_bytes()));

    Ok(build_app_state(user_repository, token_service))
}

/// Wire use cases from the given adapters
///
/// Separated from [`create_app_state`] so tests can supply in-memory adapters.
pub fn build_app_state(
    user_repository: Arc<dyn UserRepository>,
    token_service: Arc<dyn TokenService>,
) -> AppState {
    AppState {
        authenticate_use_case: Arc::new(AuthenticateUseCase::new(
            user_repository.clone(),
            token_service.clone(),
        )),
        login_use_case: Arc::new(LoginUseCase::new(user_repository.clone(), token_service)),
        register_use_case: Arc::new(RegisterUseCase::new(user_repository.clone())),
        create_user_use_case: Arc::new(CreateUserUseCase::new(user_repository.clone())),
        get_user_use_case: Arc::new(GetUserUseCase::new(user_repository.clone())),
        list_users_use_case: Arc::new(ListUsersUseCase::new(user_repository.clone())),
        update_user_use_case: Arc::new(UpdateUserUseCase::new(user_repository)),
    }
}
