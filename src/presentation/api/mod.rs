//! API handlers
//!
//! HTTP request handlers organized by domain.

pub mod auth;
pub mod health;
pub mod users;

pub use auth::auth_routes;
pub use health::health_routes;
pub use users::user_routes;
