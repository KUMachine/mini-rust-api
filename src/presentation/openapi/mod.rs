//! OpenAPI documentation
//!
//! Swagger/OpenAPI specification generation using utoipa.

use crate::app::auth::{AuthToken, LoginCommand, RegisterCommand};
use crate::app::user::{CreateUserCommand, UpdateUserCommand, UserResponse};
use utoipa::OpenApi;

/// API Documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::presentation::api::users::list_users,
        crate::presentation::api::users::create_user,
        crate::presentation::api::users::update_user,
        crate::presentation::api::users::get_user,
        crate::presentation::api::health::health_check,
        crate::presentation::api::auth::login,
        crate::presentation::api::auth::register
    ),
    components(
        schemas(UserResponse, CreateUserCommand, UpdateUserCommand, LoginCommand, RegisterCommand, AuthToken)
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "auth", description = "Authentication endpoints")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}
