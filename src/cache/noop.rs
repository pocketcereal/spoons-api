//! No-op cache service implementation for testing or cache-disabled scenarios.

use super::service::CacheService;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::time::Duration;

/// No-op cache service that performs no caching operations.
///
/// Useful for:
/// - Testing with cache disabled
/// - Development environments where caching is not needed
/// - Debugging cache-related issues
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpCacheService;

impl NoOpCacheService {
    /// Creates a new no-op cache service.
    pub fn new() -> Self {
        Self
    }
}

impl CacheService for NoOpCacheService {
    async fn get<T: DeserializeOwned + Send>(&self, _key: &str) -> Option<T> {
        None
    }

    async fn set<T: Serialize + Send + Sync>(&self, _key: &str, _value: &T, _ttl: Duration) {
        // No-op
    }

    async fn remove(&self, _key: &str) {
        // No-op
    }

    async fn clear(&self) {
        // No-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        value: String,
    }

    #[tokio::test]
    async fn test_noop_get_returns_none() {
        let cache = NoOpCacheService::new();
        let result: Option<TestData> = cache.get("any_key").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_noop_set_does_nothing() {
        let cache = NoOpCacheService::new();
        let data = TestData {
            value: "test".to_string(),
        };

        cache.set("key", &data, Duration::from_secs(60)).await;

        // Get should still return None
        let result: Option<TestData> = cache.get("key").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_noop_remove_does_nothing() {
        let cache = NoOpCacheService::new();
        cache.remove("any_key").await;
        // Should not panic
    }

    #[tokio::test]
    async fn test_noop_clear_does_nothing() {
        let cache = NoOpCacheService::new();
        cache.clear().await;
        // Should not panic
    }

    #[tokio::test]
    async fn test_noop_default() {
        let cache = NoOpCacheService;
        let result: Option<TestData> = cache.get("key").await;
        assert_eq!(result, None);
    }
}
