//! In-memory cache service implementation with TTL and LRU eviction.

use super::service::CacheService;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cache entry containing serialized data and expiration metadata.
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Serialized data as bytes
    data: Vec<u8>,
    /// When this entry expires
    expires_at: Instant,
    /// When this entry was last accessed (for LRU eviction)
    last_accessed: Instant,
}

impl CacheEntry {
    /// Creates a new cache entry.
    fn new(data: Vec<u8>, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            data,
            expires_at: now + ttl,
            last_accessed: now,
        }
    }

    /// Checks if the entry has expired.
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// Updates the last accessed timestamp.
    fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }
}

/// In-memory cache service with TTL expiration and LRU eviction.
///
/// # Features
/// - Thread-safe with `Arc<RwLock<_>>`
/// - Automatic TTL-based expiration
/// - LRU eviction when max entries exceeded
/// - JSON serialization/deserialization via serde_json
#[derive(Debug, Clone)]
pub struct InMemoryCacheService {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    max_entries: usize,
}

impl InMemoryCacheService {
    /// Creates a new in-memory cache service.
    ///
    /// # Arguments
    /// * `max_entries` - Maximum number of entries before LRU eviction occurs
    pub fn new(max_entries: usize) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            max_entries,
        }
    }

    /// Evicts the least recently used entry from the cache.
    fn evict_lru(store: &mut HashMap<String, CacheEntry>) {
        if let Some((lru_key, _)) = store
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            store.remove(&lru_key);
            tracing::debug!(key = %lru_key, "Evicted LRU entry from cache");
        }
    }
}

impl CacheService for InMemoryCacheService {
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> Option<T> {
        let mut store = self.store.write().await;

        if let Some(entry) = store.get_mut(key) {
            if entry.is_expired() {
                store.remove(key);
                tracing::debug!(key = %key, "Cache entry expired");
                return None;
            }

            entry.touch();

            match serde_json::from_slice::<T>(&entry.data) {
                Ok(value) => {
                    tracing::debug!(key = %key, "Cache hit");
                    Some(value)
                }
                Err(e) => {
                    tracing::warn!(key = %key, error = %e, "Failed to deserialize cache entry");
                    store.remove(key);
                    None
                }
            }
        } else {
            tracing::debug!(key = %key, "Cache miss");
            None
        }
    }

    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T, ttl: Duration) {
        let data = match serde_json::to_vec(value) {
            Ok(d) => d,
            Err(e) => {
                tracing::error!(key = %key, error = %e, "Failed to serialize cache value");
                return;
            }
        };

        let mut store = self.store.write().await;

        // Evict LRU entry if at capacity and key doesn't exist
        if store.len() >= self.max_entries && !store.contains_key(key) {
            Self::evict_lru(&mut store);
        }

        let entry = CacheEntry::new(data, ttl);
        store.insert(key.to_string(), entry);
        tracing::debug!(key = %key, ttl_secs = ?ttl.as_secs(), "Cache entry set");
    }

    async fn remove(&self, key: &str) {
        let mut store = self.store.write().await;
        if store.remove(key).is_some() {
            tracing::debug!(key = %key, "Cache entry removed");
        }
    }

    async fn clear(&self) {
        let mut store = self.store.write().await;
        let count = store.len();
        store.clear();
        tracing::debug!(count = count, "Cache cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tokio::time::sleep;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        value: String,
        number: i32,
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = InMemoryCacheService::new(10);
        let data = TestData {
            value: "test".to_string(),
            number: 42,
        };

        cache.set("key1", &data, Duration::from_secs(60)).await;
        let result: Option<TestData> = cache.get("key1").await;

        assert_eq!(result, Some(data));
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = InMemoryCacheService::new(10);
        let result: Option<TestData> = cache.get("nonexistent").await;

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = InMemoryCacheService::new(10);
        let data = TestData {
            value: "test".to_string(),
            number: 42,
        };

        // Set with very short TTL
        cache.set("key1", &data, Duration::from_millis(50)).await;

        // Should exist immediately
        let result: Option<TestData> = cache.get("key1").await;
        assert_eq!(result, Some(data.clone()));

        // Wait for expiration
        sleep(Duration::from_millis(100)).await;

        // Should be expired
        let result: Option<TestData> = cache.get("key1").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let cache = InMemoryCacheService::new(3);
        let data1 = TestData {
            value: "data1".to_string(),
            number: 1,
        };
        let data2 = TestData {
            value: "data2".to_string(),
            number: 2,
        };
        let data3 = TestData {
            value: "data3".to_string(),
            number: 3,
        };
        let data4 = TestData {
            value: "data4".to_string(),
            number: 4,
        };

        // Fill cache to capacity
        cache.set("key1", &data1, Duration::from_secs(60)).await;
        sleep(Duration::from_millis(10)).await; // Ensure different timestamps
        cache.set("key2", &data2, Duration::from_secs(60)).await;
        sleep(Duration::from_millis(10)).await;
        cache.set("key3", &data3, Duration::from_secs(60)).await;

        // Access key1 to make it more recently used than key2
        let _: Option<TestData> = cache.get("key1").await;

        // Adding a 4th entry should evict key2 (least recently used)
        cache.set("key4", &data4, Duration::from_secs(60)).await;

        // key2 should be evicted
        let result: Option<TestData> = cache.get("key2").await;
        assert_eq!(result, None);

        // key1 should still exist
        let result: Option<TestData> = cache.get("key1").await;
        assert_eq!(result, Some(data1));

        // key3 should still exist
        let result: Option<TestData> = cache.get("key3").await;
        assert_eq!(result, Some(data3));

        // key4 should exist
        let result: Option<TestData> = cache.get("key4").await;
        assert_eq!(result, Some(data4));
    }

    #[tokio::test]
    async fn test_remove() {
        let cache = InMemoryCacheService::new(10);
        let data = TestData {
            value: "test".to_string(),
            number: 42,
        };

        cache.set("key1", &data, Duration::from_secs(60)).await;
        cache.remove("key1").await;

        let result: Option<TestData> = cache.get("key1").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_clear() {
        let cache = InMemoryCacheService::new(10);
        let data = TestData {
            value: "test".to_string(),
            number: 42,
        };

        cache.set("key1", &data, Duration::from_secs(60)).await;
        cache.set("key2", &data, Duration::from_secs(60)).await;
        cache.set("key3", &data, Duration::from_secs(60)).await;

        cache.clear().await;

        let result1: Option<TestData> = cache.get("key1").await;
        let result2: Option<TestData> = cache.get("key2").await;
        let result3: Option<TestData> = cache.get("key3").await;

        assert_eq!(result1, None);
        assert_eq!(result2, None);
        assert_eq!(result3, None);
    }

    #[tokio::test]
    async fn test_update_existing_entry() {
        let cache = InMemoryCacheService::new(3);
        let data1 = TestData {
            value: "original".to_string(),
            number: 1,
        };
        let data2 = TestData {
            value: "updated".to_string(),
            number: 2,
        };

        cache.set("key1", &data1, Duration::from_secs(60)).await;
        cache.set("key1", &data2, Duration::from_secs(60)).await;

        let result: Option<TestData> = cache.get("key1").await;
        assert_eq!(result, Some(data2));

        // Should still have only 1 entry
        let store = cache.store.read().await;
        assert_eq!(store.len(), 1);
    }
}
