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
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),

    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),

    #[error("User not found")]
    UserNotFound,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User with email {0} already exists")]
    EmailAlreadyExists(String),

    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Type alias for application results
pub type AppResult<T> = Result<T, ApplicationError>;
