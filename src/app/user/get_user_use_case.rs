use super::UserResponse;
use crate::app::caller_context::CallerContext;
use crate::app::errors::{AppResult, ApplicationError};
use crate::domain::shared::UserId;
use crate::domain::user::UserRepository;
use std::sync::Arc;

/// GetUserUseCase - handles retrieving a single user
pub struct GetUserUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl GetUserUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, user_id: i32, caller: &CallerContext) -> AppResult<UserResponse> {
        // Authorization: admin can view any user, regular users only their own
        if !caller.can_access_user(user_id) {
            return Err(ApplicationError::Forbidden(
                "You can only view your own profile".to_string(),
            ));
        }

        let user_id = UserId::from(user_id);

        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(ApplicationError::UserNotFound)?;

        Ok(UserResponse::from_domain(&user))
    }
}
