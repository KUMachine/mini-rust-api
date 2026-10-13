use super::registration::{NewUser, register_user};
use super::{CreateUserCommand, UserResponse};
use crate::app::caller_context::CallerContext;
use crate::app::errors::{AppResult, ApplicationError};
use crate::domain::user::UserRepository;
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

        register_user(
            self.user_repository.as_ref(),
            NewUser {
                email: command.email,
                password: command.password,
                first_name: command.first_name,
                last_name: command.last_name,
                age: command.age,
            },
        )
        .await
    }
}
