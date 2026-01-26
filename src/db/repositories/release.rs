//! Release repository for database operations.

use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::db::models::{NewReleaseGroupRow, NewReleaseRow, ReleaseGroupRow, ReleaseRow};
use crate::db::schema::{release_groups, releases};
use crate::db::{get_conn, parse_uuid, validate_batch_size, DbPool};
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
        let uuid = parse_uuid(id)?;
        let min_cached_at = Utc::now() - Duration::seconds(cache_ttl_seconds);
        let mut conn = get_conn(pool).await?;

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
            .map_err(|e| AppError::Database(format!("Failed to get cached release: {}", e)))?;

        Ok(result.map(|(release_row, release_group_row)| {
            release_row.into_release(release_group_row.map(Into::into))
        }))
    }

    /// Get a release by ID (regardless of cache expiry).
    #[allow(dead_code)] // Used in integration tests
    pub async fn get_by_id(pool: &DbPool, id: &str) -> Result<Option<Release>> {
        let uuid = parse_uuid(id)?;
        let mut conn = get_conn(pool).await?;

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
            .map_err(|e| AppError::Database(format!("Failed to get release: {}", e)))?;

        Ok(result.map(|(release_row, release_group_row)| {
            release_row.into_release(release_group_row.map(Into::into))
        }))
    }

    /// Get multiple releases by their IDs.
    ///
    /// # Errors
    /// Returns an error if the batch size exceeds the maximum allowed (100).
    pub async fn get_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Release>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        validate_batch_size(ids.len())?;

        let mut conn = get_conn(pool).await?;

        let results: Vec<(ReleaseRow, Option<ReleaseGroupRow>)> = releases::table
            .left_join(release_groups::table)
            .filter(releases::id.eq_any(ids))
            .select((
                ReleaseRow::as_select(),
                Option::<ReleaseGroupRow>::as_select(),
            ))
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Database(format!("Failed to get releases by IDs: {}", e)))?;

        Ok(results
            .into_iter()
            .map(|(release_row, release_group_row)| {
                release_row.into_release(release_group_row.map(Into::into))
            })
            .collect())
    }

    /// Upsert a release (insert or update).
    ///
    /// Release group and release upserts are performed atomically within a transaction.
    pub async fn upsert(pool: &DbPool, release: &Release) -> Result<()> {
        let mut conn = get_conn(pool).await?;

        let new_rg = release
            .release_group
            .as_ref()
            .map(|rg| {
                NewReleaseGroupRow::try_from(rg)
                    .map_err(|e| AppError::Database(format!("Invalid release group UUID: {}", e)))
            })
            .transpose()?;

        let new_release = NewReleaseRow::try_from(release)
            .map_err(|e| AppError::Database(format!("Invalid release UUID: {}", e)))?;

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                if let Some(ref new_rg) = new_rg {
                    diesel::insert_into(release_groups::table)
                        .values(new_rg)
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
                        .execute(conn)
                        .await
                        .map_err(|e| AppError::Database(format!("Failed to upsert release group: {}", e)))?;
                }

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
                    .execute(conn)
                    .await
                    .map_err(|e| AppError::Database(format!("Failed to upsert release: {}", e)))?;

                Ok(())
            })
        })
        .await
    }

    /// Upsert multiple releases using batch operations.
    pub async fn upsert_many(pool: &DbPool, releases_list: &[Release]) -> Result<()> {
        if releases_list.is_empty() {
            return Ok(());
        }

        let mut conn = get_conn(pool).await?;

        let new_release_groups: Vec<NewReleaseGroupRow> = releases_list
            .iter()
            .filter_map(|r| r.release_group.as_ref())
            .filter_map(|rg| {
                NewReleaseGroupRow::try_from(rg).map_err(|e| {
                    tracing::warn!(release_group_id = %rg.id, error = %e, "Skipping release group with invalid UUID");
                    e
                }).ok()
            })
            .collect();

        if !new_release_groups.is_empty() {
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
        }

        let new_releases: Vec<NewReleaseRow> = releases_list
            .iter()
            .filter_map(|r| {
                NewReleaseRow::try_from(r).map_err(|e| {
                    tracing::warn!(release_id = %r.id, error = %e, "Skipping release with invalid UUID");
                    e
                }).ok()
            })
            .collect();

        if new_releases.is_empty() {
            return Ok(());
        }

        diesel::insert_into(releases::table)
            .values(&new_releases)
            .on_conflict(releases::id)
            .do_update()
            .set((
                releases::title.eq(diesel::dsl::sql::<diesel::sql_types::Text>("excluded.title")),
                releases::status.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.status")),
                releases::release_date.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.release_date")),
                releases::country.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.country")),
                releases::barcode.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.barcode")),
                releases::disambiguation.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.disambiguation")),
                releases::release_group_id.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>>("excluded.release_group_id")),
                releases::updated_at.eq(Utc::now()),
                releases::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Database(format!("Failed to batch upsert releases: {}", e)))?;

        Ok(())
    }
}
