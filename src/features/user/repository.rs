use super::{Email, User};
use crate::features::shared::UserId;
use async_trait::async_trait;
use thiserror::Error;

/// Repository errors
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("User not found")]
    NotFound,

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

/// UserRepository trait - defines the contract for user persistence
/// This trait lives in the domain layer, implementations are in infrastructure
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find a user by their unique identifier
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError>;

    /// Find a user by their email address
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError>;

    /// Save a user (insert if new, update if existing)
    async fn save(&self, user: &mut User) -> Result<(), RepositoryError>;

    /// Check if a user exists with the given email
    async fn exists_with_email(&self, email: &Email) -> Result<bool, RepositoryError>;

    /// List users with pagination
    async fn list(
        &self,
        page: u64,
        rows_per_page: u64,
    ) -> Result<(Vec<User>, u64), RepositoryError>;
}
