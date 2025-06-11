use serde::{Deserialize, Serialize};
use crate::entity::users;

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub first_name: String,
    pub last_name: String,
    pub age: i32,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub age: i32,
}

impl From<users::Model> for UserResponse {
    fn from(user: users::Model) -> Self {
        Self {
            id: user.id,
            first_name: user.first_name,
            last_name: user.last_name,
            age: user.age,
        }
    }
}
