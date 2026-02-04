use super::{UpdateUserCommand, UserResponse};
use crate::app::errors::{AppResult, ApplicationError};
use crate::domain::shared::UserId;
use crate::domain::user::{Email, UserRepository};
use std::sync::Arc;

/// UpdateUserUseCase - handles updating an existing user
pub struct UpdateUserUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl UpdateUserUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(
        &self,
        user_id: i32,
        command: UpdateUserCommand,
    ) -> AppResult<UserResponse> {
        let user_id = UserId::from(user_id);

        // Find the user
        let mut user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(ApplicationError::UserNotFound)?;

        // Parse and validate email (domain validation)
        let new_email = Email::try_from(command.email)?;

        // Domain logic: update email if different
        user.change_email(new_email)?;

        // Domain logic: update profile
        user.update_profile(command.first_name, command.last_name, command.age)?;

        // Persist changes
        self.user_repository.save(&mut user).await?;

        // Convert to response DTO
        Ok(UserResponse::from_domain(&user))
    }
}
