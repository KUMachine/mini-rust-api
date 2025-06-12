use crate::models::user::UserResponse;
use crate::validators::user::CreateUserRequest;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::users::list_users,
        crate::routes::users::create_user,
        crate::routes::health::health_check
    ),
    components(
        schemas(UserResponse, CreateUserRequest)
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "users", description = "User management endpoints")
    )
)]
pub struct ApiDoc;
