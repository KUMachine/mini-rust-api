use crate::models::user::UserResponse;
use crate::validators::user::CreateUserRequest;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::users::list_users,
        crate::routes::users::create_user
    ),
    components(
        schemas(UserResponse, CreateUserRequest)
    ),
    tags(
        (name = "users", description = "User management endpoints")
    )
)]
pub struct ApiDoc;
