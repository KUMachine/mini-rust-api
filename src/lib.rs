pub mod auth;
pub mod config;
pub mod docs;
pub mod entity;
pub mod errors;
pub mod extractors;
pub mod middleware;
pub mod models;
pub mod pagination;
pub mod response;
pub mod routes;
pub mod validators;

// Re-export commonly used items for convenience
pub use auth::{Claims, auth_middleware, auth_routes};
pub use config::Config;
pub use config::database;
pub use errors::AppError;
pub use middleware::{cors::cors_layer, tracing::tracing_layer};
pub use models::user::UserResponse;
pub use response::ApiResponse;

use sea_orm::DbConn;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbConn>,
}
