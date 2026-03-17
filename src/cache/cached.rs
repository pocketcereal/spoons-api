//! Generic cache-first helpers for source API operations.
//!
//! These functions implement the common pattern:
//! 1. Check cache (DB lookup with TTL)
//! 2. On hit → return cached data
//! 3. On miss → fetch from upstream API
//! 4. Fire-and-forget cache write
//! 5. Return API result

use std::future::Future;

use crate::error::{AppError, Result};

/// Seconds-based cache TTL to avoid accidentally mixing units.
pub type CacheTtlSeconds = i64;

/// Cache-first get: check cache, fetch from API on miss, fire-and-forget write.
///
/// The `cache_write` closure receives a reference to the fetched value,
/// allowing it to clone only what's needed for the background task.
pub async fn cached_get<T, CacheFut, ApiFut, WriteF>(
    cache_check: CacheFut,
    api_fetch: ApiFut,
    cache_write: WriteF,
    entity_name: &str,
    cache_key: &str,
) -> Result<T>
where
    CacheFut: Future<Output = Result<Option<T>>>,
    ApiFut: Future<Output = Result<T>>,
    WriteF: FnOnce(&T),
{
    match cache_check.await {
        Ok(Some(cached)) => {
            tracing::debug!(entity = entity_name, key = cache_key, "Cache hit");
            return Ok(cached);
        }
        Ok(None) => {}
        Err(e) => {
            tracing::warn!(entity = entity_name, key = cache_key, error = %e, "Cache check failed, treating as miss");
        }
    }

    tracing::debug!(entity = entity_name, key = cache_key, "Cache miss, fetching from API");
    let result = api_fetch.await?;
    cache_write(&result);
    Ok(result)
}

/// Cache-first get that converts `NotFound` errors to `None`.
///
/// Use this for single-entity lookups where the upstream API returns
/// a not-found error rather than an empty result.
pub async fn cached_get_optional<T, CacheFut, ApiFut, WriteF>(
    cache_check: CacheFut,
    api_fetch: ApiFut,
    cache_write: WriteF,
    entity_name: &str,
    cache_key: &str,
) -> Result<Option<T>>
where
    CacheFut: Future<Output = Result<Option<T>>>,
    ApiFut: Future<Output = Result<T>>,
    WriteF: FnOnce(&T),
{
    match cached_get(cache_check, api_fetch, cache_write, entity_name, cache_key).await {
        Ok(value) => Ok(Some(value)),
        Err(AppError::NotFound(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Cache-first search: check cache, fetch from API on miss, fire-and-forget write.
///
/// The `cache_write` closure receives a reference to the fetched results,
/// allowing it to clone only what's needed for the background task.
pub async fn cached_search<T, CacheFut, ApiFut, WriteF>(
    cache_check: CacheFut,
    api_fetch: ApiFut,
    cache_write: WriteF,
    entity_name: &str,
    query: &str,
) -> Result<Vec<T>>
where
    CacheFut: Future<Output = Result<Option<Vec<T>>>>,
    ApiFut: Future<Output = Result<Vec<T>>>,
    WriteF: FnOnce(&[T]),
{
    match cache_check.await {
        Ok(Some(cached)) => {
            tracing::debug!(entity = entity_name, query = query, "Search cache hit");
            return Ok(cached);
        }
        Ok(None) => {}
        Err(e) => {
            tracing::warn!(entity = entity_name, query = query, error = %e, "Search cache check failed, treating as miss");
        }
    }

    tracing::debug!(entity = entity_name, query = query, "Search cache miss, fetching from API");
    let results = api_fetch.await?;
    cache_write(&results);
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

    #[tokio::test]
    async fn cached_get_returns_cached_value_on_hit() {
        let result = cached_get(
            async { Ok(Some(42)) },
            async { panic!("API should not be called on cache hit") },
            |_: &i32| panic!("cache_write should not be called on cache hit"),
            "test",
            "key",
        )
        .await
        .unwrap();

        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn cached_get_fetches_api_on_miss() {
        let write_called = Arc::new(AtomicBool::new(false));
        let write_called_clone = write_called.clone();

        let result = cached_get(
            async { Ok(None) },
            async { Ok(99) },
            move |val: &i32| {
                assert_eq!(*val, 99);
                write_called_clone.store(true, Ordering::Relaxed);
            },
            "test",
            "key",
        )
        .await
        .unwrap();

        assert_eq!(result, 99);
        assert!(write_called.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn cached_get_degrades_on_cache_check_error() {
        let write_called = Arc::new(AtomicBool::new(false));
        let write_called_clone = write_called.clone();

        let result = cached_get(
            async { Err(AppError::Database("db down".to_string())) },
            async { Ok(1) },
            move |_: &i32| {
                write_called_clone.store(true, Ordering::Relaxed);
            },
            "test",
            "key",
        )
        .await
        .unwrap();

        assert_eq!(result, 1);
        assert!(write_called.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn cached_get_propagates_api_error() {
        let result: Result<i32> = cached_get(
            async { Ok(None) },
            async { Err(AppError::Server("api down".to_string())) },
            |_: &i32| {},
            "test",
            "key",
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cached_get_optional_returns_none_on_not_found() {
        let result = cached_get_optional(
            async { Ok(None) },
            async { Err(AppError::NotFound("not found".to_string())) },
            |_: &i32| {},
            "test",
            "key",
        )
        .await
        .unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn cached_get_optional_returns_some_on_success() {
        let result = cached_get_optional(
            async { Ok(None) },
            async { Ok(42) },
            |_: &i32| {},
            "test",
            "key",
        )
        .await
        .unwrap();

        assert_eq!(result, Some(42));
    }

    #[tokio::test]
    async fn cached_get_optional_propagates_non_not_found_errors() {
        let result: Result<Option<i32>> = cached_get_optional(
            async { Ok(None) },
            async { Err(AppError::Server("server error".to_string())) },
            |_: &i32| {},
            "test",
            "key",
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn cached_search_returns_cached_on_hit() {
        let result = cached_search(
            async { Ok(Some(vec![1, 2, 3])) },
            async { panic!("API should not be called on cache hit") },
            |_: &[i32]| panic!("cache_write should not be called on cache hit"),
            "test",
            "query",
        )
        .await
        .unwrap();

        assert_eq!(result, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn cached_search_fetches_api_on_miss() {
        let write_called = Arc::new(AtomicBool::new(false));
        let write_called_clone = write_called.clone();

        let result = cached_search(
            async { Ok(None) },
            async { Ok(vec![4, 5]) },
            move |vals: &[i32]| {
                assert_eq!(vals, &[4, 5]);
                write_called_clone.store(true, Ordering::Relaxed);
            },
            "test",
            "query",
        )
        .await
        .unwrap();

        assert_eq!(result, vec![4, 5]);
        assert!(write_called.load(Ordering::Relaxed));
    }
}
