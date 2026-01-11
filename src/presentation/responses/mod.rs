//! Response types for the API
//!
//! Standardized response wrappers following JSON:API format.

mod api_response;
mod pagination;

pub use api_response::{ApiErrorResponse, ApiResponse, JsonApiError, JsonApiErrorSource, Meta};
pub use pagination::PaginationRequest;
