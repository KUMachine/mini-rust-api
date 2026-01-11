//! Health check API handler
//!
//! Simple endpoint to verify the API is running.

use crate::presentation::state::AppState;
use axum::{Router, routing::get};

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "API Health", body = String)
    ),
    tag = "health"
)]
pub async fn health_check() -> String {
    "Ok".to_string()
}

/// Create health check routes
pub fn health_routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}
