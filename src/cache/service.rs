//! Cache service trait definition.

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::time::Duration;

/// Trait for cache service implementations.
///
/// Provides a common interface for different caching strategies
/// (in-memory, Redis, no-op, etc.).
pub trait CacheService: Send + Sync {
    /// Retrieves a value from the cache by key.
    ///
    /// Returns `None` if the key doesn't exist or has expired.
    fn get<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> impl std::future::Future<Output = Option<T>> + Send;

    /// Stores a value in the cache with a time-to-live duration.
    ///
    /// # Arguments
    /// * `key` - The cache key
    /// * `value` - The value to store
    /// * `ttl` - Time-to-live duration before expiration
    fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> impl std::future::Future<Output = ()> + Send;

    /// Removes a value from the cache by key.
    fn remove(&self, key: &str) -> impl std::future::Future<Output = ()> + Send;

    /// Clears all entries from the cache.
    fn clear(&self) -> impl std::future::Future<Output = ()> + Send;
}
