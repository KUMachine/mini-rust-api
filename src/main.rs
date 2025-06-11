mod db;
mod entity;
mod config;
mod routes;
mod models;
mod response;
mod errors;
mod middleware;

use axum::{
    Router,
    routing::{get},
};
use sea_orm::{ActiveModelTrait, DbConn, EntityTrait};
use serde::{Deserialize, Serialize};
use std::{sync::Arc};
use tracing_subscriber::fmt::layer;
use crate::config::Config;
use crate::middleware::{ cors_layer, tracing_layer};

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
        .merge( routes::users::routes())
        .route("/health", get(|| async { "Okay!" }))
        .layer(cors_layer())
        .layer(tracing_layer())
        .with_state(state);

    let address = format!("{}:{}", config.server_host, config.server_port);

    let listener = tokio::net::TcpListener::bind(&address).await.expect("Failed to bind the address");

    // println!("Server running at http://{}", address);
    tracing::debug!("🚀 Server listening at http://{}", address);
    axum::serve(listener, app).await.unwrap();
}
