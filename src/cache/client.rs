//! Redis client with graceful failure handling.

#![allow(dead_code)]

use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Cache configuration.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Redis connection URL.
    pub url: String,
    /// Default TTL for cached items (in seconds).
    pub default_ttl: u64,
    /// Whether caching is enabled.
    pub enabled: bool,
}

impl CacheConfig {
    /// Create config from environment variables.
    pub fn from_env() -> Self {
        let url = std::env::var("SPOONS_REDIS_URL")
            .or_else(|_| std::env::var("REDIS_URL"))
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let default_ttl = std::env::var("SPOONS_CACHE_TTL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300);

        let enabled = std::env::var("SPOONS_CACHE_DISABLED")
            .map(|v| v != "true" && v != "1")
            .unwrap_or(true);

        Self {
            url,
            default_ttl,
            enabled,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            default_ttl: 300,
            enabled: true,
        }
    }
}

/// Cache options for a specific operation.
#[derive(Debug, Clone, Default)]
pub struct CacheOptions {
    /// TTL in seconds (None uses default).
    pub ttl: Option<u64>,
    /// Cache key prefix.
    pub prefix: Option<String>,
}

impl CacheOptions {
    /// Create options with a specific TTL.
    pub fn with_ttl(ttl: u64) -> Self {
        Self {
            ttl: Some(ttl),
            prefix: None,
        }
    }

    /// Create options with a key prefix.
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            ttl: None,
            prefix: Some(prefix.into()),
        }
    }
}

/// Redis cache client that fails gracefully.
#[derive(Clone)]
pub struct CacheClient {
    connection: Arc<RwLock<Option<ConnectionManager>>>,
    config: CacheConfig,
}

impl CacheClient {
    /// Create a new cache client.
    pub async fn new(config: CacheConfig) -> Self {
        let connection = if config.enabled {
            match Self::connect(&config.url).await {
                Ok(conn) => {
                    tracing::info!("Redis connection established");
                    Some(conn)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to connect to Redis, caching disabled");
                    None
                }
            }
        } else {
            tracing::info!("Redis caching disabled by configuration");
            None
        };

        Self {
            connection: Arc::new(RwLock::new(connection)),
            config,
        }
    }

    /// Connect to Redis.
    async fn connect(url: &str) -> Result<ConnectionManager, redis::RedisError> {
        let client = Client::open(url)?;
        ConnectionManager::new(client).await
    }

    /// Check if the cache is available.
    pub async fn is_available(&self) -> bool {
        self.connection.read().await.is_some()
    }

    /// Get a value from the cache.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let guard = self.connection.read().await;
        let conn = (*guard).as_ref()?;

        let mut conn = conn.clone();
        match conn.get::<_, Option<String>>(key).await {
            Ok(Some(data)) => match serde_json::from_str(&data) {
                Ok(value) => {
                    tracing::debug!(key = %key, "Cache hit");
                    Some(value)
                }
                Err(e) => {
                    tracing::warn!(key = %key, error = %e, "Failed to deserialize cached value");
                    None
                }
            },
            Ok(None) => {
                tracing::debug!(key = %key, "Cache miss");
                None
            }
            Err(e) => {
                tracing::warn!(key = %key, error = %e, "Redis get failed");
                None
            }
        }
    }

    /// Set a value in the cache.
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, options: Option<CacheOptions>) {
        let guard = self.connection.read().await;
        let Some(ref conn) = *guard else {
            return;
        };

        let ttl = options
            .as_ref()
            .and_then(|o| o.ttl)
            .unwrap_or(self.config.default_ttl);

        let data = match serde_json::to_string(value) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!(key = %key, error = %e, "Failed to serialize value for cache");
                return;
            }
        };

        let mut conn = conn.clone();
        if let Err(e) = conn
            .set_ex::<_, _, ()>(key, data, Duration::from_secs(ttl).as_secs())
            .await
        {
            tracing::warn!(key = %key, error = %e, "Redis set failed");
        } else {
            tracing::debug!(key = %key, ttl = %ttl, "Value cached");
        }
    }

    /// Delete a value from the cache.
    pub async fn delete(&self, key: &str) {
        let guard = self.connection.read().await;
        let Some(ref conn) = *guard else {
            return;
        };

        let mut conn = conn.clone();
        if let Err(e) = conn.del::<_, ()>(key).await {
            tracing::warn!(key = %key, error = %e, "Redis delete failed");
        } else {
            tracing::debug!(key = %key, "Cache entry deleted");
        }
    }

    /// Get or set a value using a factory function.
    pub async fn get_or_set<T, F, Fut>(
        &self,
        key: &str,
        factory: F,
        options: Option<CacheOptions>,
    ) -> Option<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Option<T>>,
    {
        // Try to get from cache first
        if let Some(cached) = self.get::<T>(key).await {
            return Some(cached);
        }

        // Call factory to get fresh value
        let value = factory().await?;

        // Store in cache
        self.set(key, &value, options).await;

        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.url, "redis://localhost:6379");
        assert_eq!(config.default_ttl, 300);
        assert!(config.enabled);
    }

    #[test]
    fn test_cache_options() {
        let opts = CacheOptions::with_ttl(60);
        assert_eq!(opts.ttl, Some(60));

        let opts = CacheOptions::with_prefix("users");
        assert_eq!(opts.prefix, Some("users".to_string()));
    }
}
