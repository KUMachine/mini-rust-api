use super::{Email, User};
use crate::domain::shared::UserId;
use async_trait::async_trait;
use thiserror::Error;

/// Repository errors
///
/// These errors are persistence-agnostic - they don't mention specific
/// storage mechanisms like databases, files, or APIs.
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Persistence failure: {0}")]
    PersistenceFailure(String),

    #[error("Entity not found")]
    NotFound,

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

pub type UserRepositoryResult<T> = Result<T, RepositoryError>;

/// UserRepository trait - defines the contract for user persistence
/// This trait lives in the domain layer, implementations are in infrastructure
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find a user by their unique identifier
    async fn find_by_id(&self, id: UserId) -> UserRepositoryResult<Option<User>>;

    /// Find a user by their email address
    async fn find_by_email(&self, email: &Email) -> UserRepositoryResult<Option<User>>;

    /// Save a user (insert if new, update if existing)
    async fn save(&self, user: &mut User) -> UserRepositoryResult<()>;

    /// Check if a user exists with the given email
    async fn exists_with_email(&self, email: &Email) -> UserRepositoryResult<bool>;

    /// List users with pagination
    async fn list(&self, page: u64, rows_per_page: u64) -> UserRepositoryResult<(Vec<User>, u64)>;
}
