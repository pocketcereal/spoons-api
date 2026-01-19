//! Release repository for database operations.

use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::models::{NewReleaseGroupRow, NewReleaseRow, ReleaseGroupRow, ReleaseRow};
use crate::db::schema::{release_groups, releases};
use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::musicbrainz::Release;

/// Repository for release database operations.
pub struct ReleaseRepository;

impl ReleaseRepository {
    /// Get a cached release by ID if not expired.
    pub async fn get_cached(
        pool: &DbPool,
        id: &str,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Release>> {
        let uuid = Uuid::parse_str(id).map_err(|e| AppError::Database(e.to_string()))?;
        let min_cached_at = Utc::now() - Duration::seconds(cache_ttl_seconds);

        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let result: Option<(ReleaseRow, Option<ReleaseGroupRow>)> = releases::table
            .left_join(release_groups::table)
            .filter(releases::id.eq(uuid))
            .filter(releases::cached_at.gt(min_cached_at))
            .select((
                ReleaseRow::as_select(),
                Option::<ReleaseGroupRow>::as_select(),
            ))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(|(release_row, release_group_row)| {
            release_row.into_release(release_group_row.map(Into::into))
        }))
    }

    /// Get a release by ID (regardless of cache expiry).
    #[allow(dead_code)] // Used in integration tests
    pub async fn get_by_id(pool: &DbPool, id: &str) -> Result<Option<Release>> {
        let uuid = Uuid::parse_str(id).map_err(|e| AppError::Database(e.to_string()))?;

        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let result: Option<(ReleaseRow, Option<ReleaseGroupRow>)> = releases::table
            .left_join(release_groups::table)
            .filter(releases::id.eq(uuid))
            .select((
                ReleaseRow::as_select(),
                Option::<ReleaseGroupRow>::as_select(),
            ))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(|(release_row, release_group_row)| {
            release_row.into_release(release_group_row.map(Into::into))
        }))
    }

    /// Get multiple releases by their IDs.
    pub async fn get_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Release>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let results: Vec<(ReleaseRow, Option<ReleaseGroupRow>)> = releases::table
            .left_join(release_groups::table)
            .filter(releases::id.eq_any(ids))
            .select((
                ReleaseRow::as_select(),
                Option::<ReleaseGroupRow>::as_select(),
            ))
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(results
            .into_iter()
            .map(|(release_row, release_group_row)| {
                release_row.into_release(release_group_row.map(Into::into))
            })
            .collect())
    }

    /// Upsert a release (insert or update).
    pub async fn upsert(pool: &DbPool, release: &Release) -> Result<()> {
        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        // First, upsert the release group if present
        if let Some(ref rg) = release.release_group {
            let new_rg =
                NewReleaseGroupRow::try_from(rg).map_err(|e| AppError::Database(e.to_string()))?;

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
        }

        // Then upsert the release
        let new_release =
            NewReleaseRow::try_from(release).map_err(|e| AppError::Database(e.to_string()))?;

        diesel::insert_into(releases::table)
            .values(&new_release)
            .on_conflict(releases::id)
            .do_update()
            .set((
                releases::title.eq(&new_release.title),
                releases::status.eq(&new_release.status),
                releases::release_date.eq(&new_release.release_date),
                releases::country.eq(&new_release.country),
                releases::barcode.eq(&new_release.barcode),
                releases::disambiguation.eq(&new_release.disambiguation),
                releases::release_group_id.eq(&new_release.release_group_id),
                releases::updated_at.eq(Utc::now()),
                releases::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Upsert multiple releases.
    pub async fn upsert_many(pool: &DbPool, releases_list: &[Release]) -> Result<()> {
        for release in releases_list {
            Self::upsert(pool, release).await?;
        }
        Ok(())
    }
}
