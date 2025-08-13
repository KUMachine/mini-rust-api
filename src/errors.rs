use crate::auth::AuthError;
use crate::response::ApiErrorResponse;
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
        let (status, errors) = match self {
            AppError::DatabaseError(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, vec![err.to_string()])
            }
            AppError::AuthError(auth_err) => {
                let message = match auth_err {
                    AuthError::WrongCredentials => "Invalid credentials",
                    AuthError::MissingCredentials => "Missing credentials",
                    AuthError::TokenCreation => "Token creation failed",
                    AuthError::InvalidToken => "Invalid token",
                };
                (
                    match auth_err {
                        AuthError::WrongCredentials | AuthError::InvalidToken => {
                            StatusCode::UNAUTHORIZED
                        }
                        AuthError::MissingCredentials => StatusCode::BAD_REQUEST,
                        AuthError::TokenCreation => StatusCode::INTERNAL_SERVER_ERROR,
                    },
                    vec![message.to_string()],
                )
            }
            AppError::ValidationError(errors) => (StatusCode::UNPROCESSABLE_ENTITY, errors),
            AppError::NotFound => (StatusCode::NOT_FOUND, vec!["Not found".to_string()]),
            AppError::Unexpected(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                vec![format!("Unexpected error: {}", err)],
            ),
        };

        let body = Json(ApiErrorResponse::new(errors));
        (status, body).into_response()
    }
}

/// Type alias for app results
pub type AppResult<T> = Result<T, AppError>;
