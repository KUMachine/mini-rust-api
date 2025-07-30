use crate::auth::AuthBody;
use crate::models::user::UserResponse;
use crate::validators::user::{CreateUserRequest, LoginRequest, RegisterRequest};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::users::list_users,
        crate::routes::users::create_user,
        crate::routes::health::health_check,
        crate::auth::handlers::login,
        crate::auth::handlers::register
    ),
    components(
        schemas(UserResponse, CreateUserRequest, LoginRequest, RegisterRequest, AuthBody)
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
