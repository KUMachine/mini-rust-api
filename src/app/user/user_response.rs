use crate::features::user::User;
use serde::Serialize;
use utoipa::ToSchema;

/// UserResponse DTO - for API responses
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub age: u8,
    pub created_at: String,
}

impl UserResponse {
    /// Convert from domain User entity
    pub fn from_domain(user: &User) -> Self {
        Self {
            id: user.id().expect("User must have an ID").value(),
            email: user.email().as_ref().to_string(),
            first_name: user.profile().first_name().to_string(),
            last_name: user.profile().last_name().to_string(),
            age: user.profile().age(),
            created_at: user.created_at().to_string(),
        }
    }
}
