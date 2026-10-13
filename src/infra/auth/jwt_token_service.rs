use crate::app::errors::ApplicationError;
use crate::app::ports::TokenService;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

/// How long an issued token stays valid
const TOKEN_TTL: Duration = Duration::days(7);

/// JWT Claims structure
///
/// Contains identity information only. Roles are looked up from the
/// database per request to ensure they are always up-to-date.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // Subject (user email)
    pub user_id: i32, // User ID
    pub iat: u64,     // Issued at
    pub exp: u64,     // Expiration time
}

/// JWT implementation of TokenService
pub struct JwtTokenService {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
}

impl JwtTokenService {
    #[must_use]
    pub fn new(secret: impl AsRef<[u8]>) -> Self {
        let secret = secret.as_ref();
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
            validation: Validation::default(),
        }
    }
}

#[async_trait]
impl TokenService for JwtTokenService {
    async fn generate_token(
        &self,
        user_id: i32,
        user_email: &str,
    ) -> Result<String, ApplicationError> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_email.to_string(),
            user_id,
            iat: now.timestamp().unsigned_abs(),
            exp: (now + TOKEN_TTL).timestamp().unsigned_abs(),
        };

        encode(&Header::default(), &claims, &self.encoding)
            .map_err(|e| ApplicationError::TokenGenerationFailed(e.to_string()))
    }

    async fn verify_token(&self, token: &str) -> Result<i32, ApplicationError> {
        decode::<Claims>(token, &self.decoding, &self.validation)
            .map(|data| data.claims.user_id)
            .map_err(|_| ApplicationError::Unauthorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generated_token_verifies() {
        let service = JwtTokenService::new("test-secret");
        let token = service.generate_token(42, "a@example.com").await.unwrap();
        assert_eq!(service.verify_token(&token).await.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_token_signed_with_other_secret_is_rejected() {
        let issuer = JwtTokenService::new("secret-one");
        let verifier = JwtTokenService::new("secret-two");
        let token = issuer.generate_token(42, "a@example.com").await.unwrap();
        assert!(matches!(
            verifier.verify_token(&token).await,
            Err(ApplicationError::Unauthorized)
        ));
    }

    #[tokio::test]
    async fn test_garbage_token_is_rejected() {
        let service = JwtTokenService::new("test-secret");
        assert!(matches!(
            service.verify_token("not-a-jwt").await,
            Err(ApplicationError::Unauthorized)
        ));
    }
}
