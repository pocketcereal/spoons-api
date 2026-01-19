//! Release group repository for database operations.

use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::models::{NewReleaseGroupRow, ReleaseGroupRow};
use crate::db::schema::release_groups;
use crate::db::DbPool;
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
        let uuid = Uuid::parse_str(id).map_err(|e| AppError::Database(e.to_string()))?;
        let min_cached_at = Utc::now() - Duration::seconds(cache_ttl_seconds);

        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let result: Option<ReleaseGroupRow> = release_groups::table
            .filter(release_groups::id.eq(uuid))
            .filter(release_groups::cached_at.gt(min_cached_at))
            .select(ReleaseGroupRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    /// Get a release group by ID (regardless of cache expiry).
    #[allow(dead_code)] // Used in integration tests
    pub async fn get_by_id(pool: &DbPool, id: &str) -> Result<Option<ReleaseGroup>> {
        let uuid = Uuid::parse_str(id).map_err(|e| AppError::Database(e.to_string()))?;

        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let result: Option<ReleaseGroupRow> = release_groups::table
            .filter(release_groups::id.eq(uuid))
            .select(ReleaseGroupRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(Into::into))
    }

    /// Get multiple release groups by their IDs.
    pub async fn get_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<ReleaseGroup>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let results: Vec<ReleaseGroupRow> = release_groups::table
            .filter(release_groups::id.eq_any(ids))
            .select(ReleaseGroupRow::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// Upsert a release group (insert or update).
    pub async fn upsert(pool: &DbPool, release_group: &ReleaseGroup) -> Result<()> {
        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let new_rg = NewReleaseGroupRow::try_from(release_group)
            .map_err(|e| AppError::Database(e.to_string()))?;

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
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Upsert multiple release groups.
    pub async fn upsert_many(pool: &DbPool, release_groups_list: &[ReleaseGroup]) -> Result<()> {
        for release_group in release_groups_list {
            Self::upsert(pool, release_group).await?;
        }
        Ok(())
    }
}
