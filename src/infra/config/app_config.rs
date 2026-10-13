//! Application configuration
//!
//! Loads configuration from environment variables using dotenvy.

use std::fmt;
use std::num::ParseIntError;

use thiserror::Error;

/// Main application configuration
#[derive(Clone, Debug)]
pub struct Config {
    pub database: Database,
    pub server: Server,
    pub auth: Auth,
}

/// Server configuration
#[derive(Clone, Debug)]
pub struct Server {
    pub host: String,
    pub port: u16,
    /// Origins allowed to make cross-origin requests (e.g. `https://app.example.com`)
    pub cors_allowed_origins: Vec<String>,
}

/// Database configuration
#[derive(Clone, Debug)]
pub struct Database {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub username: String,
    pub password: String,
}

/// Authentication configuration
#[derive(Clone)]
pub struct Auth {
    pub jwt_secret: String,
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Auth")
            .field("jwt_secret", &"[REDACTED]")
            .finish()
    }
}

/// Configuration loading errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{name} is required")]
    MissingEnv {
        name: &'static str,
        #[source]
        source: dotenvy::Error,
    },

    #[error("{name} must not be empty")]
    EmptyEnv { name: &'static str },

    #[error("{name} must be a valid port, got `{value}`")]
    InvalidPort {
        name: &'static str,
        value: String,
        #[source]
        source: ParseIntError,
    },
}

impl Database {
    /// Build the database connection URL
    #[must_use]
    pub fn build_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name
        )
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database: Database {
                username: fetch_env("DATABASE__USERNAME")?,
                password: fetch_env("DATABASE__PASSWORD")?,
                host: fetch_env("DATABASE__HOST")?,
                port: parse_port("DATABASE__PORT", fetch_env("DATABASE__PORT")?)?,
                name: fetch_env("DATABASE__NAME")?,
            },
            server: Server {
                host: fetch_env_with_default("SERVER__HOST", "0.0.0.0"),
                port: parse_port(
                    "SERVER__PORT",
                    fetch_env_with_default("SERVER__PORT", "3000"),
                )?,
                cors_allowed_origins: parse_list(&fetch_env_with_default(
                    "SERVER__CORS_ALLOWED_ORIGINS",
                    "",
                )),
            },
            auth: Auth {
                jwt_secret: jwt_secret(fetch_env("JWT_SECRET")?)?,
            },
        })
    }
}

/// HS256 keys should be at least as long as the hash output (256 bits)
const MIN_RECOMMENDED_JWT_SECRET_LEN: usize = 32;

fn jwt_secret(secret: String) -> Result<String, ConfigError> {
    if secret.trim().is_empty() {
        return Err(ConfigError::EmptyEnv { name: "JWT_SECRET" });
    }

    if secret.len() < MIN_RECOMMENDED_JWT_SECRET_LEN {
        tracing::warn!(
            "JWT_SECRET is shorter than {MIN_RECOMMENDED_JWT_SECRET_LEN} bytes; use a longer random secret"
        );
    }

    Ok(secret)
}

fn fetch_env(name: &'static str) -> Result<String, ConfigError> {
    dotenvy::var(name).map_err(|source| ConfigError::MissingEnv { name, source })
}

fn fetch_env_with_default(name: &'static str, default: &str) -> String {
    dotenvy::var(name).unwrap_or_else(|_| default.to_string())
}

fn parse_port(name: &'static str, value: String) -> Result<u16, ConfigError> {
    value
        .parse::<u16>()
        .map_err(|source| ConfigError::InvalidPort {
            name,
            value,
            source,
        })
}

fn parse_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list() {
        assert_eq!(
            parse_list(" http://a.com, http://b.com ,,"),
            vec!["http://a.com", "http://b.com"]
        );
        assert!(parse_list("").is_empty());
    }

    #[test]
    fn test_jwt_secret_must_not_be_empty() {
        assert!(matches!(
            jwt_secret("  ".to_string()),
            Err(ConfigError::EmptyEnv { name: "JWT_SECRET" })
        ));
        assert!(jwt_secret("a-sufficiently-long-random-secret-value".to_string()).is_ok());
    }
}
