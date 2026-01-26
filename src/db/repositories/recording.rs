//! Recording repository for database operations.

use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::models::{NewRecordingRow, RecordingRow};
use crate::db::schema::recordings;
use crate::db::{DbPool, db_error, get_conn, parse_uuid, validate_batch_size};
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
        let uuid = parse_uuid(id)?;
        let min_cached_at = Utc::now() - Duration::seconds(cache_ttl_seconds);
        let mut conn = get_conn(pool).await?;

        let result: Option<RecordingRow> = recordings::table
            .filter(recordings::id.eq(uuid))
            .filter(recordings::cached_at.gt(min_cached_at))
            .select(RecordingRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get cached recording"))?;

        Ok(result.map(Into::into))
    }

    /// Get a recording by ID (regardless of cache expiry).
    #[allow(dead_code)] // Used in integration tests
    pub async fn get_by_id(pool: &DbPool, id: &str) -> Result<Option<Recording>> {
        let uuid = parse_uuid(id)?;
        let mut conn = get_conn(pool).await?;

        let result: Option<RecordingRow> = recordings::table
            .filter(recordings::id.eq(uuid))
            .select(RecordingRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get recording"))?;

        Ok(result.map(Into::into))
    }

    /// Get multiple recordings by their IDs.
    ///
    /// # Errors
    /// Returns an error if the batch size exceeds the maximum allowed (100).
    pub async fn get_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Recording>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        validate_batch_size(ids.len())?;

        let mut conn = get_conn(pool).await?;

        let results: Vec<RecordingRow> = recordings::table
            .filter(recordings::id.eq_any(ids))
            .select(RecordingRow::as_select())
            .load(&mut conn)
            .await
            .map_err(db_error("Failed to get recordings by IDs"))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// Upsert a recording (insert or update).
    pub async fn upsert(pool: &DbPool, recording: &Recording) -> Result<()> {
        let mut conn = get_conn(pool).await?;

        let new_recording = NewRecordingRow::try_from(recording)
            .map_err(|e| AppError::Database(format!("Invalid recording UUID: {}", e)))?;

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
            .map_err(db_error("Failed to upsert recording"))?;

        Ok(())
    }

    /// Upsert multiple recordings using batch insert.
    pub async fn upsert_many(pool: &DbPool, recordings_list: &[Recording]) -> Result<()> {
        if recordings_list.is_empty() {
            return Ok(());
        }

        let new_recordings: Vec<NewRecordingRow> = recordings_list
            .iter()
            .filter_map(|r| {
                NewRecordingRow::try_from(r).map_err(|e| {
                    tracing::warn!(recording_id = %r.id, error = %e, "Skipping recording with invalid UUID");
                    e
                }).ok()
            })
            .collect();

        if new_recordings.is_empty() {
            return Ok(());
        }

        let mut conn = get_conn(pool).await?;

        diesel::insert_into(recordings::table)
            .values(&new_recordings)
            .on_conflict(recordings::id)
            .do_update()
            .set((
                recordings::title.eq(diesel::dsl::sql::<diesel::sql_types::Text>(
                    "excluded.title",
                )),
                recordings::length_ms.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Int8>,
                >("excluded.length_ms")),
                recordings::disambiguation.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.disambiguation")),
                recordings::video.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Bool>,
                >("excluded.video")),
                recordings::updated_at.eq(Utc::now()),
                recordings::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to batch upsert recordings"))?;

        Ok(())
    }
}
