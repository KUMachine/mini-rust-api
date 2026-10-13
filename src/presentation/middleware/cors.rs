//! CORS middleware configuration
//!
//! Cross-Origin Resource Sharing configuration for the API.

use crate::infra::Config;
use axum::http::{HeaderValue, Method, header, header::InvalidHeaderValue};
use thiserror::Error;
use tower_http::cors::CorsLayer;

/// CORS configuration errors
#[derive(Debug, Error)]
pub enum CorsLayerError {
    #[error("invalid cors origin `{origin}`")]
    InvalidOrigin {
        origin: String,
        #[source]
        source: InvalidHeaderValue,
    },
}

/// Create CORS layer allowing the configured origins
///
/// With no origins configured, cross-origin requests are not allowed.
pub fn cors_layer(config: &Config) -> Result<CorsLayer, CorsLayerError> {
    let origins = config
        .server
        .cors_allowed_origins
        .iter()
        .map(|origin| {
            origin
                .parse::<HeaderValue>()
                .map_err(|source| CorsLayerError::InvalidOrigin {
                    origin: origin.clone(),
                    source,
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]))
}
