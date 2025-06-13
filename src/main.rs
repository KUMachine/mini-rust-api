mod config;
mod db;
mod docs;
mod entity;
mod errors;
mod middleware;
mod models;
mod response;
mod routes;
mod validators;

use crate::config::Config;
use crate::middleware::{cors_layer, tracing_layer};
use axum::{Router, routing::get};
use sea_orm::DbConn;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
struct AppState {
    db: Arc<DbConn>,
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    let db = Arc::new(
        db::connection::connect()
            .await
            .expect("Failed to connect to database"),
    );

    let state = AppState { db };

    let app = Router::new()
        .merge(routes::users::routes())
        .route("/health", get(routes::health::health_check))
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", docs::ApiDoc::openapi()))
        .layer(cors_layer())
        .layer(tracing_layer())
        .with_state(state);

    let address = format!("{}:{}", config.server_host, config.server_port);

    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("Failed to bind the address");

    tracing::debug!("🚀 Server listening at http://{}", address);
    axum::serve(listener, app).await.unwrap();
}
