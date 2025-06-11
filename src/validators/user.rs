use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, message = "First name is required"))]
    pub first_name: String,

    #[validate(length(min = 1, message = "Last name is required"))]
    pub last_name: String,

    #[validate(range(min = 0, message = "Age must be non-negative"))]
    pub age: i32,
}
