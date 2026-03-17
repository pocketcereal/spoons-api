use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;

use crate::config::DatabaseConfig;
use crate::error::{AppError, Result};

pub type DbPool = Pool<AsyncPgConnection>;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub url: String,
    pub max_connections: usize,
}

impl DbConfig {
    pub fn from_env() -> Result<Self> {
        let url = std::env::var("SPOONS_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .map_err(|_| {
                AppError::Config("DATABASE_URL or SPOONS_DATABASE_URL must be set".to_string())
            })?;

        let max_connections = match std::env::var("SPOONS_DB_MAX_CONNECTIONS") {
            Ok(s) => s.parse().map_err(|_| {
                AppError::Config(format!(
                    "SPOONS_DB_MAX_CONNECTIONS must be a valid number, got: '{}'",
                    s
                ))
            })?,
            Err(_) => 10,
        };

        Ok(Self {
            url,
            max_connections,
        })
    }

    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            max_connections: 10,
        }
    }
}

impl TryFrom<&DatabaseConfig> for DbConfig {
    type Error = AppError;

    /// Resolves URL from config, then SPOONS_DATABASE_URL, then DATABASE_URL env vars.
    fn try_from(config: &DatabaseConfig) -> Result<Self> {
        let url = config
            .url
            .clone()
            .or_else(|| std::env::var("SPOONS_DATABASE_URL").ok())
            .or_else(|| std::env::var("DATABASE_URL").ok())
            .ok_or_else(|| AppError::Config("DATABASE_URL must be set".to_string()))?;

        Ok(Self {
            url,
            max_connections: config.max_connections,
        })
    }
}

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
