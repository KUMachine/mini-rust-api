pub mod login_use_case;
pub mod register_use_case;

pub use login_use_case::LoginUseCase;
pub use register_use_case::RegisterUseCase;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Command for login
#[derive(Debug, Clone, Deserialize, ToSchema, Validate)]
pub struct LoginCommand {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

/// Command for registration
#[derive(Debug, Clone, Deserialize, ToSchema, Validate)]
pub struct RegisterCommand {
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

/// Authentication result with token
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthToken {
    pub access_token: String,
    pub token_type: String,
}

impl AuthToken {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}
