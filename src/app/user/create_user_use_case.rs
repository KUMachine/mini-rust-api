use super::{CreateUserCommand, UserResponse};
use crate::app::errors::{AppResult, ApplicationError};
use crate::domain::user::{Email, User, UserRepository};
use std::sync::Arc;

/// CreateUserUseCase - handles creating a new user
pub struct CreateUserUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl CreateUserUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, command: CreateUserCommand) -> AppResult<UserResponse> {
        // Parse and validate email (domain validation)
        let email = Email::try_from(command.email.clone())?;

        // Check if user already exists
        if self.user_repository.exists_with_email(&email).await? {
            return Err(ApplicationError::EmailAlreadyExists(command.email));
        }

        // Create user entity (domain logic)
        let mut user = User::register(
            email,
            command.password,
            command.first_name,
            command.last_name,
            command.age,
        )?;

        // Persist the user
        self.user_repository.save(&mut user).await?;

        // Convert to response DTO
        Ok(UserResponse::from_domain(&user))
    }
}
