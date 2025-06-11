mod db;
mod entity;
mod config;
mod routes;
mod models;
mod response;
mod errors;

use axum::{
    Router,
    routing::{get},
};
use sea_orm::{ActiveModelTrait, DbConn, EntityTrait};
use serde::{Deserialize, Serialize};
use std::{sync::Arc};
use crate::config::Config;

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
        .with_state(state);

    let address = format!("{}:{}", config.server_host, config.server_port);

    let listener = tokio::net::TcpListener::bind(&address).await.expect("Failed to bind the address");

    println!("🚀 Server running at http://{}", address);
    axum::serve(listener, app).await.unwrap();
}
