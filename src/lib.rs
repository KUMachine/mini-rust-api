pub mod config;
pub mod db;
pub mod docs;
pub mod entity;
pub mod errors;
pub mod middleware;
pub mod models;
pub mod pagination;
pub mod response;
pub mod routes;
pub mod validators;

// Re-export commonly used items for convenience
pub use config::Config;
pub use db::connection;
pub use errors::AppError;
pub use middleware::{cors_layer, tracing_layer};
pub use models::user::UserResponse;
pub use response::ApiResponse;

use sea_orm::DbConn;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbConn>,
}
