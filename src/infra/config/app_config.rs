//! Application configuration
//!
//! Loads configuration from environment variables using dotenvy.

use std::fmt;
use std::num::ParseIntError;

use thiserror::Error;

/// Main application configuration
#[derive(Clone, Debug)]
pub struct Config {
    pub storage: Storage,
    pub server: Server,
    pub auth: Auth,
}

/// Where data is persisted, selected by `STORAGE`
#[derive(Clone, Debug)]
pub enum Storage {
    /// `STORAGE=postgres` (default): PostgreSQL configured by `DATABASE__*`
    Postgres(Database),
    /// `STORAGE=memory`: process-local, no database needed, data is lost on restart
    Memory,
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
    MissingEnv { name: &'static str },

    #[error("{name} must not be empty")]
    EmptyEnv { name: &'static str },

    #[error("{name} must be a valid port, got `{value}`")]
    InvalidPort {
        name: &'static str,
        value: String,
        #[source]
        source: ParseIntError,
    },

    #[error("STORAGE must be `postgres` or `memory`, got `{0}`")]
    InvalidStorage(String),
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
    /// Load configuration from environment variables (and `.env`, if present)
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();
        Self::from_lookup(|name| dotenvy::var(name).ok())
    }

    /// Load configuration from any variable source
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let env = Env(&lookup);

        let storage = match env.or("STORAGE", "postgres").as_str() {
            "postgres" => Storage::Postgres(Database {
                username: env.required("DATABASE__USERNAME")?,
                password: env.required("DATABASE__PASSWORD")?,
                host: env.required("DATABASE__HOST")?,
                port: parse_port("DATABASE__PORT", env.required("DATABASE__PORT")?)?,
                name: env.required("DATABASE__NAME")?,
            }),
            "memory" => Storage::Memory,
            other => return Err(ConfigError::InvalidStorage(other.to_string())),
        };

        Ok(Self {
            storage,
            server: Server {
                host: env.or("SERVER__HOST", "0.0.0.0"),
                port: parse_port("SERVER__PORT", env.or("SERVER__PORT", "3000"))?,
                cors_allowed_origins: parse_list(&env.or("SERVER__CORS_ALLOWED_ORIGINS", "")),
            },
            auth: Auth {
                jwt_secret: jwt_secret(env.required("JWT_SECRET")?)?,
            },
        })
    }
}

/// Typed access to a variable lookup
struct Env<'a, F: Fn(&str) -> Option<String>>(&'a F);

impl<F: Fn(&str) -> Option<String>> Env<'_, F> {
    fn required(&self, name: &'static str) -> Result<String, ConfigError> {
        (self.0)(name).ok_or(ConfigError::MissingEnv { name })
    }

    fn or(&self, name: &str, default: &str) -> String {
        (self.0)(name).unwrap_or_else(|| default.to_string())
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
    use std::collections::HashMap;

    const SECRET: &str = "a-sufficiently-long-random-secret-value";

    fn load(vars: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let vars: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Config::from_lookup(|name| vars.get(name).cloned())
    }

    #[test]
    fn test_postgres_storage_is_default_and_requires_database_vars() {
        assert!(matches!(
            load(&[("JWT_SECRET", SECRET)]),
            Err(ConfigError::MissingEnv {
                name: "DATABASE__USERNAME"
            })
        ));

        let config = load(&[
            ("JWT_SECRET", SECRET),
            ("DATABASE__USERNAME", "postgres"),
            ("DATABASE__PASSWORD", "secret"),
            ("DATABASE__HOST", "localhost"),
            ("DATABASE__PORT", "5432"),
            ("DATABASE__NAME", "app"),
        ])
        .unwrap();
        assert!(matches!(config.storage, Storage::Postgres(ref db) if db.port == 5432));
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn test_memory_storage_needs_no_database_vars() {
        let config = load(&[("STORAGE", "memory"), ("JWT_SECRET", SECRET)]).unwrap();
        assert!(matches!(config.storage, Storage::Memory));
    }

    #[test]
    fn test_unknown_storage_is_rejected() {
        assert!(matches!(
            load(&[("STORAGE", "redis"), ("JWT_SECRET", SECRET)]),
            Err(ConfigError::InvalidStorage(value)) if value == "redis"
        ));
    }

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
        assert!(jwt_secret(SECRET.to_string()).is_ok());
    }
}
