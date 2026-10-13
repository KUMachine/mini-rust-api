//! Caller context extractor
//!
//! Extracts the `CallerContext` inserted into request extensions by the auth middleware.

use crate::app::{ApplicationError, CallerContext};
use axum::http::request::Parts;
use axum_core::extract::FromRequestParts;

impl<S> FromRequestParts<S> for CallerContext
where
    S: Send + Sync,
{
    type Rejection = ApplicationError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CallerContext>()
            .cloned()
            .ok_or(ApplicationError::Unauthorized)
    }
}
