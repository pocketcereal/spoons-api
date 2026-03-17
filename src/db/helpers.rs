use chrono::{DateTime, Duration, Utc};
use std::future::Future;
use uuid::Uuid;

use crate::db::DbPool;
use crate::error::{AppError, Result};
pub fn min_cached_at(cache_ttl_seconds: i64) -> Result<DateTime<Utc>> {
    let duration = Duration::try_seconds(cache_ttl_seconds).ok_or_else(|| {
        AppError::Config(format!(
            "Invalid cache TTL value: {} seconds",
            cache_ttl_seconds
        ))
    })?;
    Ok(Utc::now() - duration)
}

pub fn parse_uuid(id: &str) -> Result<Uuid> {
    Uuid::parse_str(id).map_err(|e| AppError::InvalidInput(format!("Invalid UUID '{}': {}", id, e)))
}

pub async fn get_conn(
    pool: &DbPool,
) -> Result<
    deadpool::managed::Object<
        diesel_async::pooled_connection::AsyncDieselConnectionManager<
            diesel_async::AsyncPgConnection,
        >,
    >,
> {
    pool.get()
        .await
        .map_err(|e| AppError::Database(format!("Failed to get connection: {}", e)))
}

pub const MAX_BATCH_SIZE: usize = 100;

pub fn validate_batch_size(len: usize) -> Result<()> {
    if len > MAX_BATCH_SIZE {
        return Err(AppError::Server(format!(
            "Batch size {} exceeds maximum of {}",
            len, MAX_BATCH_SIZE
        )));
    }
    Ok(())
}

/// Reduces boilerplate for `.map_err(|e| AppError::Database(format!("context: {}", e)))`.
pub fn db_error(context: &str) -> impl FnOnce(diesel::result::Error) -> AppError + '_ {
    move |e| AppError::Database(format!("{}: {}", context, e))
}

/// Fire-and-forget cache write: spawns a task, logs errors, never propagates.
pub fn spawn_cache_task<F, Fut>(entity_name: &'static str, cache_fn: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<()>> + Send,
{
    tokio::spawn(async move {
        if let Err(e) = cache_fn().await {
            tracing::warn!(error = %e, "Failed to cache {}", entity_name);
        }
    });
}
