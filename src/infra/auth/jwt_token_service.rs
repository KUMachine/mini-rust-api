use crate::app::errors::ApplicationError;
use crate::app::ports::TokenService;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // Subject (user email)
    pub user_id: i32, // User ID
    pub exp: usize,   // Expiration time
}

/// Keys for JWT encoding/decoding
struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

/// JWT implementation of TokenService
pub struct JwtTokenService;

impl JwtTokenService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_decoding_key() -> &'static DecodingKey {
        &KEYS.decoding
    }
}

impl Default for JwtTokenService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TokenService for JwtTokenService {
    async fn generate_token(
        &self,
        user_id: i32,
        user_email: &str,
    ) -> Result<String, ApplicationError> {
        let exp = (Utc::now() + Duration::days(7)).timestamp() as usize;
        let claims = Claims {
            sub: user_email.to_string(),
            user_id,
            exp,
        };

        encode(&Header::default(), &claims, &KEYS.encoding)
            .map_err(|e| ApplicationError::TokenGenerationFailed(e.to_string()))
    }
}
