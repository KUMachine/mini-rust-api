use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sea_orm::DbErr;
use thiserror::Error;
use crate::response::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DbErr),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found")]
    NotFound,

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::ValidationError(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Unexpected(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(ApiErrorResponse::error(message));
        (status, body).into_response()
    }
}

/// Type alias for app results
pub type AppResult<T> = Result<T, AppError>;
