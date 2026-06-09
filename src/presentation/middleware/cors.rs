//! CORS middleware configuration
//!
//! Cross-Origin Resource Sharing configuration for the API.

use crate::infra::Config;
use axum::http::{HeaderValue, Method, header::InvalidHeaderValue};
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

/// Create CORS layer with configured origins
pub fn cors_layer(config: &Config) -> Result<CorsLayer, CorsLayerError> {
    let address = format!("{}:{}", config.server.host, config.server.port);
    let origin =
        address
            .parse::<HeaderValue>()
            .map_err(|source| CorsLayerError::InvalidOrigin {
                origin: address,
                source,
            })?;

    Ok(CorsLayer::new().allow_origin(origin).allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
    ]))
}
