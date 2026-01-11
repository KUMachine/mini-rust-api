//! Presentation Layer
//!
//! This layer handles all HTTP/API concerns:
//! - API handlers (controllers)
//! - HTTP extractors
//! - HTTP middleware
//! - Response types
//! - OpenAPI documentation
//! - Application state

pub mod api;
pub mod extractors;
pub mod middleware;
pub mod openapi;
pub mod responses;
pub mod state;

pub use state::AppState;
