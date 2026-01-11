//! Database connection configuration
//!
//! Handles SeaORM database connection pooling and configuration.

use super::Config;
use sea_orm::{ConnectOptions, Database, DbConn, DbErr};
use std::time::Duration;

/// Database connection configuration builder
pub struct DatabaseConfig {
    url: String,
    max_connections: u32,
    min_connections: u32,
    connect_timeout: Duration,
    idle_timeout: Duration,
    max_lifetime: Duration,
}

impl DatabaseConfig {
    /// Create a new database configuration with the given URL
    pub fn new(url: String) -> Self {
        Self {
            url,
            max_connections: 100,
            min_connections: 5,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
        }
    }

    /// Set the maximum number of connections
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Set the minimum number of connections
    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = min;
        self
    }

    /// Set the connection timeout
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set the idle timeout
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Set the maximum connection lifetime
    pub fn max_lifetime(mut self, lifetime: Duration) -> Self {
        self.max_lifetime = lifetime;
        self
    }

    /// Connect to the database with the configured options
    pub async fn connect(self) -> Result<DbConn, DbErr> {
        let mut opts = ConnectOptions::new(&self.url);
        opts.max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .connect_timeout(self.connect_timeout)
            .idle_timeout(self.idle_timeout)
            .max_lifetime(self.max_lifetime);

        Database::connect(opts).await
    }
}

/// Connect to the database using environment configuration
pub async fn connect() -> Result<DbConn, DbErr> {
    let config = Config::from_env();

    DatabaseConfig::new(config.database.build_url())
        .max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .connect()
        .await
}
