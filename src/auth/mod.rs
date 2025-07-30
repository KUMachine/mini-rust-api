pub mod handlers;
pub mod middleware;
pub mod service;
pub mod types;

pub use handlers::auth_routes;
pub use middleware::{auth_middleware, get_claims_from_request};
pub use service::{create_jwt_token, hash_password, verify_password};
pub use types::{AuthBody, AuthError, Claims};
