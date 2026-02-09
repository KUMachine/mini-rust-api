pub mod login_use_case;
pub mod register_use_case;

pub use login_use_case::LoginUseCase;
pub use register_use_case::RegisterUseCase;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Command for login
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct LoginCommand {
    pub email: String,
    pub password: String,
}

/// Command for registration
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RegisterCommand {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
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
