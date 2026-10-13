//! Presentation layer error handling
//!
//! This module handles the translation of application errors to HTTP responses.
//! All HTTP concerns (status codes, JSON formatting) are isolated here.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::app::ApplicationError;
use crate::domain::user::errors::DomainError;

use super::responses::{ApiErrorResponse, JsonApiError};

/// Detail returned for server-side failures; the real cause is logged, not exposed
const INTERNAL_ERROR_DETAIL: &str = "An internal error occurred";

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        use StatusCode as S;

        let (status, code, title, detail) = match self {
            ApplicationError::DomainError(err) => return domain_error_response(err),
            ApplicationError::RepositoryError(err) => {
                tracing::error!(error = %err, "repository error");
                (
                    S::INTERNAL_SERVER_ERROR,
                    "PERSISTENCE_ERROR",
                    "Persistence Error",
                    INTERNAL_ERROR_DETAIL.to_string(),
                )
            }
            ApplicationError::UserNotFound => (
                S::NOT_FOUND,
                "USER_NOT_FOUND",
                "User Not Found",
                "The requested user was not found".to_string(),
            ),
            ApplicationError::InvalidCredentials => (
                S::UNAUTHORIZED,
                "INVALID_CREDENTIALS",
                "Invalid Credentials",
                "The provided credentials are incorrect".to_string(),
            ),
            ApplicationError::EmailAlreadyExists(email) => (
                S::CONFLICT,
                "EMAIL_ALREADY_EXISTS",
                "Email Already Exists",
                format!("A user with email '{email}' already exists"),
            ),
            ApplicationError::TokenGenerationFailed(msg) => {
                tracing::error!(error = %msg, "token generation failed");
                (
                    S::INTERNAL_SERVER_ERROR,
                    "TOKEN_GENERATION_FAILED",
                    "Token Generation Failed",
                    INTERNAL_ERROR_DETAIL.to_string(),
                )
            }
            ApplicationError::Unauthorized => (
                S::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Unauthorized",
                "A valid bearer token is required to access this resource".to_string(),
            ),
            ApplicationError::Forbidden(msg) => (S::FORBIDDEN, "FORBIDDEN", "Forbidden", msg),
            ApplicationError::ValidationError(msg) => (
                S::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR",
                "Validation Error",
                msg,
            ),
            ApplicationError::MissingUserId => {
                tracing::error!("persisted user is missing its id");
                (
                    S::INTERNAL_SERVER_ERROR,
                    "MISSING_USER_ID",
                    "Missing User ID",
                    INTERNAL_ERROR_DETAIL.to_string(),
                )
            }
        };

        error_response(status, code, title, detail)
    }
}

/// Convert domain errors to HTTP responses
fn domain_error_response(err: DomainError) -> Response {
    use StatusCode as S;

    let (status, code, title) = match &err {
        DomainError::InvalidEmail(_) => (S::BAD_REQUEST, "INVALID_EMAIL", "Invalid Email"),
        DomainError::PasswordTooShort => {
            (S::BAD_REQUEST, "PASSWORD_TOO_SHORT", "Password Too Short")
        }
        DomainError::PasswordTooWeak => (S::BAD_REQUEST, "PASSWORD_TOO_WEAK", "Password Too Weak"),
        DomainError::UserTooYoung => (S::BAD_REQUEST, "USER_TOO_YOUNG", "User Too Young"),
        DomainError::InvalidAge => (S::BAD_REQUEST, "INVALID_AGE", "Invalid Age"),
        DomainError::EmptyFirstName => (S::BAD_REQUEST, "EMPTY_FIRST_NAME", "Empty First Name"),
        DomainError::EmptyLastName => (S::BAD_REQUEST, "EMPTY_LAST_NAME", "Empty Last Name"),
        DomainError::InvalidCredentials => (
            S::UNAUTHORIZED,
            "INVALID_CREDENTIALS",
            "Invalid Credentials",
        ),
        DomainError::PasswordHashingFailed => {
            tracing::error!("password hashing failed");
            return error_response(
                S::INTERNAL_SERVER_ERROR,
                "PASSWORD_HASHING_FAILED",
                "Password Hashing Failed",
                INTERNAL_ERROR_DETAIL,
            );
        }
        DomainError::EmailAlreadyExists(_) => {
            (S::CONFLICT, "EMAIL_ALREADY_EXISTS", "Email Already Exists")
        }
    };

    // Domain error messages are written for end users, so they double as the detail
    error_response(status, code, title, capitalize(&err.to_string()))
}

fn error_response(
    status: StatusCode,
    code: &str,
    title: &str,
    detail: impl Into<String>,
) -> Response {
    let error = JsonApiError::new(status.as_u16(), code, title).with_detail(detail);
    (status, Json(ApiErrorResponse::from_single_error(error))).into_response()
}

fn capitalize(message: &str) -> String {
    let mut chars = message.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::repository::RepositoryError;

    async fn body_json(response: Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_repository_error_details_are_not_exposed() {
        let error = ApplicationError::RepositoryError(RepositoryError::PersistenceFailure(
            "connection refused to 10.0.0.5:5432".to_string(),
        ));
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = body_json(response).await;
        assert_eq!(body["errors"][0]["detail"], INTERNAL_ERROR_DETAIL);
        assert!(!body.to_string().contains("10.0.0.5"));
    }

    #[tokio::test]
    async fn test_domain_error_uses_its_message_as_detail() {
        let response = ApplicationError::DomainError(DomainError::UserTooYoung).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = body_json(response).await;
        assert_eq!(body["errors"][0]["code"], "USER_TOO_YOUNG");
        assert_eq!(
            body["errors"][0]["detail"],
            "User must be at least 18 years old"
        );
    }
}
