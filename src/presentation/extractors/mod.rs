//! Custom Axum extractors
//!
//! Provides validation-aware extractors for request handling.

mod caller_context;
mod validated_json;
mod validated_pagination;

pub use validated_json::{JsonBody, ValidatedJson};
pub use validated_pagination::{PaginationQuery, ValidatedPagination};
