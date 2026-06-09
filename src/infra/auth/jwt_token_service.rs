use crate::app::errors::ApplicationError;
use crate::app::ports::TokenService;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// JWT Claims structure
///
/// Contains identity information only. Roles are looked up from the
/// database per request to ensure they are always up-to-date.
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

/// JWT implementation of TokenService
pub struct JwtTokenService {
    keys: Arc<Keys>,
}

impl JwtTokenService {
    #[must_use]
    pub fn new(secret: impl AsRef<[u8]>) -> Self {
        Self {
            keys: Arc::new(Keys::new(secret.as_ref())),
        }
    }

    #[must_use]
    pub fn decoding_key(&self) -> &DecodingKey {
        &self.keys.decoding
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

        encode(&Header::default(), &claims, &self.keys.encoding)
            .map_err(|e| ApplicationError::TokenGenerationFailed(e.to_string()))
    }
}
