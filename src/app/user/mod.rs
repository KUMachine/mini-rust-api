pub mod create_user_use_case;
pub mod get_user_use_case;
pub mod list_users_use_case;
pub mod update_user_use_case;
pub mod user_response;

pub use create_user_use_case::CreateUserUseCase;
pub use get_user_use_case::GetUserUseCase;
pub use list_users_use_case::ListUsersUseCase;
pub use update_user_use_case::UpdateUserUseCase;
pub use user_response::UserResponse;

use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

/// Command for creating a user
#[derive(Debug, Clone, Deserialize, ToSchema, Validate)]
pub struct CreateUserCommand {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1))]
    pub first_name: String,
    #[validate(length(min = 1))]
    pub last_name: String,
    #[validate(range(min = 18, max = 150))]
    pub age: u8,
}

/// Command for updating a user
#[derive(Debug, Clone, Deserialize, ToSchema, Validate)]
pub struct UpdateUserCommand {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub first_name: String,
    #[validate(length(min = 1))]
    pub last_name: String,
    #[validate(range(min = 18, max = 150))]
    pub age: u8,
}

/// Query for listing users
#[derive(Debug, Clone)]
pub struct ListUsersQuery {
    pub page: u64,
    pub rows_per_page: u64,
}
