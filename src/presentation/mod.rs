//! Presentation Layer
//!
//! This layer handles all HTTP/API concerns:
//! - API handlers (controllers)
//! - HTTP extractors
//! - HTTP middleware
//! - Response types
//! - Error responses (HTTP translation)
//! - OpenAPI documentation
//! - Application state

pub mod api;
pub mod errors;
pub mod extractors;
pub mod middleware;
pub mod openapi;
pub mod responses;
pub mod state;

pub use state::AppState;
