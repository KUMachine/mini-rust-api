use super::{CreateUserCommand, UserResponse};
use crate::app::caller_context::CallerContext;
use crate::app::errors::{AppResult, ApplicationError};
use crate::domain::user::{Email, User, UserRepository};
use std::sync::Arc;

/// CreateUserUseCase - handles creating a new user (admin only)
pub struct CreateUserUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl CreateUserUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(
        &self,
        command: CreateUserCommand,
        caller: &CallerContext,
    ) -> AppResult<UserResponse> {
        // Authorization: only admins can create users via this endpoint
        if !caller.is_admin() {
            return Err(ApplicationError::Forbidden(
                "Only administrators can create users".to_string(),
            ));
        }

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
