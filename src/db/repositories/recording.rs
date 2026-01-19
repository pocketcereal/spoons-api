//! Recording repository for database operations.

use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::models::{NewRecordingRow, RecordingRow};
use crate::db::schema::recordings;
use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::musicbrainz::Recording;

/// Repository for recording database operations.
pub struct RecordingRepository;

impl RecordingRepository {
    /// Get a cached recording by ID if not expired.
    pub async fn get_cached(
        pool: &DbPool,
        id: &str,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Recording>> {
        let uuid = Uuid::parse_str(id).map_err(|e| AppError::Database(e.to_string()))?;
        let min_cached_at = Utc::now() - Duration::seconds(cache_ttl_seconds);

        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let result: Option<RecordingRow> = recordings::table
            .filter(recordings::id.eq(uuid))
            .filter(recordings::cached_at.gt(min_cached_at))
            .select(RecordingRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    /// Get a recording by ID (regardless of cache expiry).
    #[allow(dead_code)] // Used in integration tests
    pub async fn get_by_id(pool: &DbPool, id: &str) -> Result<Option<Recording>> {
        let uuid = Uuid::parse_str(id).map_err(|e| AppError::Database(e.to_string()))?;

        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let result: Option<RecordingRow> = recordings::table
            .filter(recordings::id.eq(uuid))
            .select(RecordingRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    /// Get multiple recordings by their IDs.
    pub async fn get_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Recording>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let results: Vec<RecordingRow> = recordings::table
            .filter(recordings::id.eq_any(ids))
            .select(RecordingRow::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// Upsert a recording (insert or update).
    pub async fn upsert(pool: &DbPool, recording: &Recording) -> Result<()> {
        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let new_recording =
            NewRecordingRow::try_from(recording).map_err(|e| AppError::Database(e.to_string()))?;

        diesel::insert_into(recordings::table)
            .values(&new_recording)
            .on_conflict(recordings::id)
            .do_update()
            .set((
                recordings::title.eq(&new_recording.title),
                recordings::length_ms.eq(&new_recording.length_ms),
                recordings::disambiguation.eq(&new_recording.disambiguation),
                recordings::video.eq(&new_recording.video),
                recordings::updated_at.eq(Utc::now()),
                recordings::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Upsert multiple recordings.
    pub async fn upsert_many(pool: &DbPool, recordings_list: &[Recording]) -> Result<()> {
        for recording in recordings_list {
            Self::upsert(pool, recording).await?;
        }
        Ok(())
    }
}
