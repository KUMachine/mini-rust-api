//! Application configuration
//!
//! Loads configuration from environment variables using dotenvy.

/// Main application configuration
#[derive(Clone, Debug)]
pub struct Config {
    pub database: Database,
    pub server: Server,
}

/// Server configuration
#[derive(Clone, Debug)]
pub struct Server {
    pub host: String,
    pub port: String,
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

impl Database {
    /// Build the database connection URL
    pub fn build_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name
        )
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            database: Database {
                username: fetch_env("DATABASE__USERNAME"),
                password: fetch_env("DATABASE__PASSWORD"),
                host: fetch_env("DATABASE__HOST"),
                port: fetch_env("DATABASE__PORT").parse::<u16>().unwrap(),
                name: fetch_env("DATABASE__NAME"),
            },
            server: Server {
                host: fetch_env_with_default("SERVER__HOST", "0.0.0.0"),
                port: fetch_env_with_default("SERVER__PORT", "3000"),
            },
        }
    }
}

fn fetch_env(var: &str) -> String {
    dotenvy::var(var).unwrap_or_else(|_| panic!("{} is required", var))
}

fn fetch_env_with_default(var: &str, default: &str) -> String {
    dotenvy::var(var).unwrap_or_else(|_| default.to_string())
}
