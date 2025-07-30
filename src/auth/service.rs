use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, encode};
use std::sync::LazyLock;

use super::types::{AuthError, Claims};

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

pub fn get_decoding_key() -> &'static DecodingKey {
    &KEYS.decoding
}

pub fn create_jwt_token(user_email: String, user_id: i32) -> Result<String, AuthError> {
    let exp = (Utc::now() + Duration::days(7)).timestamp() as usize;
    let claims = Claims {
        sub: user_email,
        user_id,
        exp,
    };

    encode(&Header::default(), &claims, &KEYS.encoding).map_err(|_| AuthError::TokenCreation)
}

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    hash(password, DEFAULT_COST).map_err(|_| AuthError::TokenCreation)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    verify(password, hash).map_err(|_| AuthError::WrongCredentials)
}
