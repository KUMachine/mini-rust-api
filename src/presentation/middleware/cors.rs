//! CORS middleware configuration
//!
//! Cross-Origin Resource Sharing configuration for the API.

use crate::infra::Config;
use axum::http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

/// Create CORS layer with configured origins
pub fn cors_layer() -> CorsLayer {
    let config = Config::from_env();
    let address = format!("{}:{}", config.server.host, config.server.port);
    CorsLayer::new()
        .allow_origin(address.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
}
