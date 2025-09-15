use crate::auth::AuthError;
use crate::response::{ApiErrorResponse, JsonApiError};
use axum::{Json, http::StatusCode, response::Response};
use axum_core::response::IntoResponse;
use sea_orm::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DbErr),

    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),

    #[error("Validation error")]
    ValidationError(Vec<String>),

    #[error("Not found")]
    NotFound,

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, api_error) = match self {
            AppError::DatabaseError(err) => {
                let error = JsonApiError::new(500, "DATABASE_ERROR", "Database Error")
                    .with_detail(format!("A database error occurred: {}", err));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorResponse::from_single_error(error),
                )
            }
            AppError::AuthError(auth_err) => {
                let (status_code, code, title, detail) = match auth_err {
                    AuthError::WrongCredentials => (
                        StatusCode::UNAUTHORIZED,
                        "WRONG_CREDENTIALS",
                        "Invalid Credentials",
                        "The provided credentials are incorrect",
                    ),
                    AuthError::MissingCredentials => (
                        StatusCode::BAD_REQUEST,
                        "MISSING_CREDENTIALS",
                        "Missing Credentials",
                        "Authentication credentials are required",
                    ),
                    AuthError::TokenCreation => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "TOKEN_CREATION_FAILED",
                        "Token Creation Failed",
                        "Failed to create authentication token",
                    ),
                    AuthError::InvalidToken => (
                        StatusCode::UNAUTHORIZED,
                        "INVALID_TOKEN",
                        "Invalid Token",
                        "The provided authentication token is invalid",
                    ),
                };
                let error =
                    JsonApiError::new(status_code.as_u16(), code, title).with_detail(detail);
                (status_code, ApiErrorResponse::from_single_error(error))
            }
            AppError::ValidationError(validation_errors) => {
                let errors: Vec<JsonApiError> = validation_errors
                    .into_iter()
                    .map(|err| {
                        JsonApiError::new(422, "VALIDATION_ERROR", "Validation Failed")
                            .with_detail(err)
                    })
                    .collect();
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorResponse::new(errors),
                )
            }
            AppError::NotFound => {
                let error = JsonApiError::new(404, "NOT_FOUND", "Resource Not Found")
                    .with_detail("The requested resource was not found");
                (
                    StatusCode::NOT_FOUND,
                    ApiErrorResponse::from_single_error(error),
                )
            }
            AppError::Unexpected(err) => {
                let error = JsonApiError::new(500, "UNEXPECTED_ERROR", "Unexpected Error")
                    .with_detail(format!("An unexpected error occurred: {}", err));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiErrorResponse::from_single_error(error),
                )
            }
        };

        let body = Json(api_error);
        (status, body).into_response()
    }
}

/// Type alias for app results
pub type AppResult<T> = Result<T, AppError>;
