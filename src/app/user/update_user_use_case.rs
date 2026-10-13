use super::registration::email_conflict;
use super::{UpdateUserCommand, UserResponse};
use crate::app::caller_context::CallerContext;
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
        caller: &CallerContext,
    ) -> AppResult<UserResponse> {
        // Authorization: admin can update any user, regular users only their own
        if !caller.can_access_user(user_id) {
            return Err(ApplicationError::Forbidden(
                "You can only update your own profile".to_string(),
            ));
        }

        let user_id = UserId::from(user_id);

        // Find the user
        let mut user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(ApplicationError::UserNotFound)?;

        // Parse and validate email (domain validation)
        let new_email = Email::try_from(command.email)?;

        // The new email must not belong to another user
        if &new_email != user.email() && self.user_repository.exists_with_email(&new_email).await? {
            return Err(ApplicationError::EmailAlreadyExists(new_email.to_string()));
        }

        // Domain logic: update email if different
        user.change_email(new_email.clone())?;

        // Domain logic: update profile
        user.update_profile(command.first_name, command.last_name, command.age)?;

        // Persist changes
        self.user_repository
            .save(&mut user)
            .await
            .map_err(|e| email_conflict(e, &new_email))?;

        // Convert to response DTO
        UserResponse::try_from_domain(&user)
    }
}
