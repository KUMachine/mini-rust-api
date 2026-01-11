use crate::features::user::errors::DomainError;
use crate::features::user::repository::RepositoryError;
use crate::presentation::responses::{ApiErrorResponse, JsonApiError};
use axum::response::{IntoResponse, Response};
use axum::{Json, http::StatusCode};
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

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        let (status, api_error) = match self {
            ApplicationError::DomainError(domain_err) => match domain_err {
                DomainError::InvalidEmail(email) => {
                    let error = JsonApiError::new(400, "INVALID_EMAIL", "Invalid Email")
                        .with_detail(format!("The email '{}' is not valid", email));
                    (
                        StatusCode::BAD_REQUEST,
                        ApiErrorResponse::from_single_error(error),
                    )
                }
                DomainError::PasswordTooShort => {
                    let error = JsonApiError::new(400, "PASSWORD_TOO_SHORT", "Password Too Short")
                        .with_detail("Password must be at least 8 characters long");
                    (
                        StatusCode::BAD_REQUEST,
                        ApiErrorResponse::from_single_error(error),
                    )
                }
                DomainError::PasswordTooWeak => {
                    let error = JsonApiError::new(400, "PASSWORD_TOO_WEAK", "Password Too Weak")
                        .with_detail("Password must contain at least one uppercase letter, one lowercase letter, and one number");
                    (
                        StatusCode::BAD_REQUEST,
                        ApiErrorResponse::from_single_error(error),
                    )
                }
                DomainError::UserTooYoung => {
                    let error = JsonApiError::new(400, "USER_TOO_YOUNG", "User Too Young")
                        .with_detail("User must be at least 18 years old");
                    (
                        StatusCode::BAD_REQUEST,
                        ApiErrorResponse::from_single_error(error),
                    )
                }
                DomainError::InvalidAge => {
                    let error = JsonApiError::new(400, "INVALID_AGE", "Invalid Age")
                        .with_detail("Age must be between 18 and 150");
                    (
                        StatusCode::BAD_REQUEST,
                        ApiErrorResponse::from_single_error(error),
                    )
                }
                DomainError::EmptyFirstName => {
                    let error = JsonApiError::new(400, "EMPTY_FIRST_NAME", "Empty First Name")
                        .with_detail("First name cannot be empty");
                    (
                        StatusCode::BAD_REQUEST,
                        ApiErrorResponse::from_single_error(error),
                    )
                }
                DomainError::EmptyLastName => {
                    let error = JsonApiError::new(400, "EMPTY_LAST_NAME", "Empty Last Name")
                        .with_detail("Last name cannot be empty");
                    (
                        StatusCode::BAD_REQUEST,
                        ApiErrorResponse::from_single_error(error),
                    )
                }
                DomainError::InvalidCredentials => {
                    let error =
                        JsonApiError::new(401, "INVALID_CREDENTIALS", "Invalid Credentials")
                            .with_detail("The provided credentials are incorrect");
                    (
                        StatusCode::UNAUTHORIZED,
                        ApiErrorResponse::from_single_error(error),
                    )
                }
                _ => {
                    let error = JsonApiError::new(400, "DOMAIN_ERROR", "Domain Error")
                        .with_detail(domain_err.to_string());
                    (
                        StatusCode::BAD_REQUEST,
                        ApiErrorResponse::from_single_error(error),
                    )
                }
            },
            ApplicationError::RepositoryError(repo_err) => {
                let error = JsonApiError::new(500, "DATABASE_ERROR", "Database Error")
                    .with_detail(format!("A database error occurred: {}", repo_err));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorResponse::from_single_error(error),
                )
            }
            ApplicationError::UserNotFound => {
                let error = JsonApiError::new(404, "USER_NOT_FOUND", "User Not Found")
                    .with_detail("The requested user was not found");
                (
                    StatusCode::NOT_FOUND,
                    ApiErrorResponse::from_single_error(error),
                )
            }
            ApplicationError::InvalidCredentials => {
                let error = JsonApiError::new(401, "INVALID_CREDENTIALS", "Invalid Credentials")
                    .with_detail("The provided credentials are incorrect");
                (
                    StatusCode::UNAUTHORIZED,
                    ApiErrorResponse::from_single_error(error),
                )
            }
            ApplicationError::EmailAlreadyExists(email) => {
                let error = JsonApiError::new(409, "EMAIL_ALREADY_EXISTS", "Email Already Exists")
                    .with_detail(format!("A user with email '{}' already exists", email));
                (
                    StatusCode::CONFLICT,
                    ApiErrorResponse::from_single_error(error),
                )
            }
            ApplicationError::TokenGenerationFailed(msg) => {
                let error =
                    JsonApiError::new(500, "TOKEN_GENERATION_FAILED", "Token Generation Failed")
                        .with_detail(msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorResponse::from_single_error(error),
                )
            }
            ApplicationError::Unauthorized => {
                let error = JsonApiError::new(401, "UNAUTHORIZED", "Unauthorized")
                    .with_detail("You are not authorized to access this resource");
                (
                    StatusCode::UNAUTHORIZED,
                    ApiErrorResponse::from_single_error(error),
                )
            }
            ApplicationError::ValidationError(msg) => {
                let error =
                    JsonApiError::new(422, "VALIDATION_ERROR", "Validation Error").with_detail(msg);
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorResponse::from_single_error(error),
                )
            }
        };

        let body = Json(api_error);
        (status, body).into_response()
    }
}
