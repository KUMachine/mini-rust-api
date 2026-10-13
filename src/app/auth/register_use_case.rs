use super::RegisterCommand;
use crate::app::errors::AppResult;
use crate::app::user::UserResponse;
use crate::app::user::registration::{NewUser, register_user};
use crate::domain::user::UserRepository;
use std::sync::Arc;

/// RegisterUseCase - handles user registration
pub struct RegisterUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl RegisterUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, command: RegisterCommand) -> AppResult<UserResponse> {
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
