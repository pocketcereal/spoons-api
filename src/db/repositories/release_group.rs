//! Release group repository for database operations.

use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::models::{NewReleaseGroupRow, ReleaseGroupRow};
use crate::db::schema::release_groups;
use crate::db::{get_conn, parse_uuid, validate_batch_size, DbPool};
use crate::error::{AppError, Result};
use crate::musicbrainz::ReleaseGroup;

/// Repository for release group database operations.
pub struct ReleaseGroupRepository;

impl ReleaseGroupRepository {
    /// Get a cached release group by ID if not expired.
    pub async fn get_cached(
        pool: &DbPool,
        id: &str,
        cache_ttl_seconds: i64,
    ) -> Result<Option<ReleaseGroup>> {
        let uuid = parse_uuid(id)?;
        let min_cached_at = Utc::now() - Duration::seconds(cache_ttl_seconds);
        let mut conn = get_conn(pool).await?;

        let result: Option<ReleaseGroupRow> = release_groups::table
            .filter(release_groups::id.eq(uuid))
            .filter(release_groups::cached_at.gt(min_cached_at))
            .select(ReleaseGroupRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(format!("Failed to get cached release group: {}", e)))?;

        Ok(result.map(Into::into))
    }

    /// Get a release group by ID (regardless of cache expiry).
    #[allow(dead_code)] // Used in integration tests
    pub async fn get_by_id(pool: &DbPool, id: &str) -> Result<Option<ReleaseGroup>> {
        let uuid = parse_uuid(id)?;
        let mut conn = get_conn(pool).await?;

        let result: Option<ReleaseGroupRow> = release_groups::table
            .filter(release_groups::id.eq(uuid))
            .select(ReleaseGroupRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(format!("Failed to get release group: {}", e)))?;

        Ok(result.map(Into::into))
    }

    /// Get multiple release groups by their IDs.
    ///
    /// # Errors
    /// Returns an error if the batch size exceeds the maximum allowed (100).
    pub async fn get_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<ReleaseGroup>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        validate_batch_size(ids.len())?;

        let mut conn = get_conn(pool).await?;

        let results: Vec<ReleaseGroupRow> = release_groups::table
            .filter(release_groups::id.eq_any(ids))
            .select(ReleaseGroupRow::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Database(format!("Failed to get release groups by IDs: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// Upsert a release group (insert or update).
    pub async fn upsert(pool: &DbPool, release_group: &ReleaseGroup) -> Result<()> {
        let mut conn = get_conn(pool).await?;

        let new_rg = NewReleaseGroupRow::try_from(release_group)
            .map_err(|e| AppError::Database(format!("Invalid release group UUID: {}", e)))?;

        diesel::insert_into(release_groups::table)
            .values(&new_rg)
            .on_conflict(release_groups::id)
            .do_update()
            .set((
                release_groups::title.eq(&new_rg.title),
                release_groups::primary_type.eq(&new_rg.primary_type),
                release_groups::secondary_types.eq(&new_rg.secondary_types),
                release_groups::first_release_date.eq(&new_rg.first_release_date),
                release_groups::disambiguation.eq(&new_rg.disambiguation),
                release_groups::updated_at.eq(Utc::now()),
                release_groups::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Database(format!("Failed to upsert release group: {}", e)))?;

        Ok(())
    }

    /// Upsert multiple release groups using batch insert.
    pub async fn upsert_many(pool: &DbPool, release_groups_list: &[ReleaseGroup]) -> Result<()> {
        if release_groups_list.is_empty() {
            return Ok(());
        }

        let new_release_groups: Vec<NewReleaseGroupRow> = release_groups_list
            .iter()
            .filter_map(|rg| {
                NewReleaseGroupRow::try_from(rg).map_err(|e| {
                    tracing::warn!(release_group_id = %rg.id, error = %e, "Skipping release group with invalid UUID");
                    e
                }).ok()
            })
            .collect();

        if new_release_groups.is_empty() {
            return Ok(());
        }

        let mut conn = get_conn(pool).await?;

        diesel::insert_into(release_groups::table)
            .values(&new_release_groups)
            .on_conflict(release_groups::id)
            .do_update()
            .set((
                release_groups::title.eq(diesel::dsl::sql::<diesel::sql_types::Text>("excluded.title")),
                release_groups::primary_type.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.primary_type")),
                release_groups::secondary_types.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>>("excluded.secondary_types")),
                release_groups::first_release_date.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.first_release_date")),
                release_groups::disambiguation.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.disambiguation")),
                release_groups::updated_at.eq(Utc::now()),
                release_groups::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Database(format!("Failed to batch upsert release groups: {}", e)))?;

        Ok(())
    }
}
