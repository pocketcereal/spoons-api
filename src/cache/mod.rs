//! Cache module providing caching services with TTL and LRU eviction.
//!
//! This module provides a trait-based caching abstraction with multiple implementations:
//! - `InMemoryCacheService`: Thread-safe in-memory cache with TTL expiration and LRU eviction
//! - `NoOpCacheService`: No-op implementation for testing or cache-disabled scenarios
//!
//! # Example
//!
//! ```
//! use spoons_api::cache::{CacheService, InMemoryCacheService};
//! use std::time::Duration;
//!
//! # async fn example() {
//! let cache = InMemoryCacheService::new(100);
//!
//! // Store a value with 60 second TTL
//! cache.set("user:123", &"John Doe", Duration::from_secs(60)).await;
//!
//! // Retrieve the value
//! let name: Option<String> = cache.get("user:123").await;
//! assert_eq!(name, Some("John Doe".to_string()));
//! # }
//! ```

mod in_memory;
mod noop;
mod service;

pub use in_memory::InMemoryCacheService;
pub use noop::NoOpCacheService;
pub use service::CacheService;
