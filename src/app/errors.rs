//! Application layer errors
//!
//! These errors represent failures in application use cases.
//! They are transport-agnostic - no HTTP concerns here.

use crate::domain::user::errors::DomainError;
use crate::domain::user::repository::RepositoryError;
use thiserror::Error;

/// Application layer errors
#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("domain error: {0}")]
    DomainError(#[from] DomainError),

    #[error("repository error: {0}")]
    RepositoryError(#[from] RepositoryError),

    #[error("user not found")]
    UserNotFound,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("user with email {0} already exists")]
    EmailAlreadyExists(String),

    #[error("token generation failed: {0}")]
    TokenGenerationFailed(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("validation error: {0}")]
    ValidationError(String),

    #[error("persisted user is missing an id")]
    MissingUserId,
}

/// Type alias for application results
pub type AppResult<T> = Result<T, ApplicationError>;
