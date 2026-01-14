//! Mini Rust API
//!
//! A Domain-Driven Design (DDD) structured REST API using Axum, SeaORM, and JWT authentication.
//!
//! # Architecture
//!
//! The codebase follows a clean hexagonal/DDD layered architecture:
//!
//! - **Domain Layer** (`domain/`): Core business logic, entities, value objects, repository traits
//! - **Application Layer** (`app/`): Use cases, DTOs, application services, ports
//! - **Infrastructure Layer** (`infra/`): Repository implementations, external services, config
//! - **Presentation Layer** (`presentation/`): HTTP handlers, middleware, extractors, responses
//! - **Bootstrap** (`bootstrap`): Dependency wiring, separated from main

pub mod app;
pub mod bootstrap;
pub mod domain;
pub mod infra;
pub mod presentation;

// Re-exports for convenience
pub use app::ApplicationError;
pub use bootstrap::create_app_state;
pub use infra::Config;
pub use presentation::AppState;
