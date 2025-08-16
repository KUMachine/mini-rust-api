use crate::config::Config;
use axum::http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

pub fn cors_layer() -> CorsLayer {
    let config: Config = Config::from_env();
    let address = format!("{}:{}", config.server_host, config.server_port);
    CorsLayer::new()
        .allow_origin(address.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET])
}
