//! Redis cache module with graceful failure handling.

mod client;

#[allow(unused_imports)]
pub use client::{CacheClient, CacheConfig, CacheOptions};
