//! Database helper functions to reduce code duplication.

use std::future::Future;
use uuid::Uuid;

use crate::db::DbPool;
use crate::error::{AppError, Result};

/// Parse a string ID to UUID, returning a database error on failure.
pub fn parse_uuid(id: &str) -> Result<Uuid> {
    Uuid::parse_str(id).map_err(|e| AppError::Database(format!("Invalid UUID '{}': {}", id, e)))
}

/// Get a connection from the pool, returning a database error on failure.
pub async fn get_conn(
    pool: &DbPool,
) -> Result<deadpool::managed::Object<diesel_async::pooled_connection::AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>> {
    pool.get()
        .await
        .map_err(|e| AppError::Database(format!("Failed to get connection: {}", e)))
}

/// Maximum batch size for bulk operations.
pub const MAX_BATCH_SIZE: usize = 100;

/// Validate that a batch size doesn't exceed the maximum.
/// Returns an error if the batch is too large, or Ok(()) if within limits.
pub fn validate_batch_size(len: usize) -> Result<()> {
    if len > MAX_BATCH_SIZE {
        return Err(AppError::Server(format!(
            "Batch size {} exceeds maximum of {}",
            len,
            MAX_BATCH_SIZE
        )));
    }
    Ok(())
}

/// Create a database error with context message.
///
/// This helper reduces boilerplate for the common pattern of:
/// `.map_err(|e| AppError::Database(format!("context: {}", e)))`
pub fn db_error(context: &str) -> impl FnOnce(diesel::result::Error) -> AppError + '_ {
    move |e| AppError::Database(format!("{}: {}", context, e))
}

/// Spawn a fire-and-forget cache operation with error logging.
///
/// This helper reduces boilerplate for the common pattern of:
/// 1. Clone data needed for async closure
/// 2. Spawn task to cache data
/// 3. Log any errors but don't propagate them
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
