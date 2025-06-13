use crate::entity::users;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub age: i32,
    #[schema(format = "date-time")]
    pub created_at: String,
}

impl From<users::Model> for UserResponse {
    fn from(user: users::Model) -> Self {
        Self {
            id: user.id,
            first_name: user.first_name,
            last_name: user.last_name,
            age: user.age,
            created_at: user.create_at.to_string(),
        }
    }
}
