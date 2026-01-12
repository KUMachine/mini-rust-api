use super::{AuthToken, LoginCommand};
use crate::app::errors::{AppResult, ApplicationError};
use crate::app::ports::TokenService;
use crate::features::user::{Email, UserRepository};
use std::sync::Arc;

/// LoginUseCase - handles user login
pub struct LoginUseCase {
    user_repository: Arc<dyn UserRepository>,
    token_service: Arc<dyn TokenService>,
}

impl LoginUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        token_service: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            user_repository,
            token_service,
        }
    }

    pub async fn execute(&self, command: LoginCommand) -> AppResult<AuthToken> {
        // Parse and validate email (domain validation)
        let email = Email::try_from(command.email)?;

        // Find user by email
        let user = self
            .user_repository
            .find_by_email(&email)
            .await?
            .ok_or(ApplicationError::InvalidCredentials)?;

        // Domain logic: authenticate user
        user.authenticate(&command.password)
            .map_err(|_| ApplicationError::InvalidCredentials)?;

        // Infrastructure concern: generate token
        let token = self
            .token_service
            .generate_token(
                user.id().ok_or(ApplicationError::UserNotFound)?.value(),
                user.email().as_ref(),
            )
            .await?;

        Ok(AuthToken::new(token))
    }
}
