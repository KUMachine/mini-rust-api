use super::{ListUsersQuery, UserResponse};
use crate::app::errors::AppResult;
use crate::domain::user::UserRepository;
use std::sync::Arc;

/// ListUsersUseCase - handles listing users with pagination
pub struct ListUsersUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl ListUsersUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, query: ListUsersQuery) -> AppResult<(Vec<UserResponse>, u64)> {
        let (users, total) = self
            .user_repository
            .list(query.page, query.rows_per_page)
            .await?;

        let user_responses: Vec<UserResponse> =
            users.iter().map(UserResponse::from_domain).collect();

        Ok((user_responses, total))
    }
}
