//! Database connection pool management.

#![allow(dead_code)]

use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;

use crate::error::{AppError, Result};

/// Type alias for the database connection pool.
pub type DbPool = Pool<AsyncPgConnection>;

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Database connection URL.
    pub url: String,
    /// Maximum number of connections in the pool.
    pub max_connections: usize,
}

impl DbConfig {
    /// Create config from environment variables.
    pub fn from_env() -> Result<Self> {
        let url = std::env::var("SPOONS_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .map_err(|_| {
                AppError::Config("DATABASE_URL or SPOONS_DATABASE_URL must be set".to_string())
            })?;

        let max_connections = std::env::var("SPOONS_DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        Ok(Self {
            url,
            max_connections,
        })
    }

    /// Create config with a specific URL.
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            max_connections: 10,
        }
    }
}

/// Create a new database connection pool.
pub fn create_pool(config: &DbConfig) -> Result<DbPool> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.url);

    Pool::builder(manager)
        .max_size(config.max_connections)
        .build()
        .map_err(|e| AppError::Config(format!("Failed to create database pool: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_config_with_url() {
        let config = DbConfig::with_url("postgres://test:test@localhost/test");
        assert_eq!(config.url, "postgres://test:test@localhost/test");
        assert_eq!(config.max_connections, 10);
    }
}
