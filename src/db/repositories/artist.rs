//! Artist repository for database operations.

use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::db::models::{AreaRow, ArtistRow, NewAreaRow, NewArtistRow};
use crate::db::schema::{areas, artists};
use crate::db::{get_conn, parse_uuid, validate_batch_size, DbPool};
use crate::error::{AppError, Result};
use crate::musicbrainz::Artist;

/// Repository for artist database operations.
pub struct ArtistRepository;

impl ArtistRepository {
    /// Get a cached artist by ID if not expired.
    pub async fn get_cached(
        pool: &DbPool,
        id: &str,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Artist>> {
        let uuid = parse_uuid(id)?;
        let min_cached_at = Utc::now() - Duration::seconds(cache_ttl_seconds);
        let mut conn = get_conn(pool).await?;

        let result: Option<(ArtistRow, Option<AreaRow>)> = artists::table
            .left_join(areas::table)
            .filter(artists::id.eq(uuid))
            .filter(artists::cached_at.gt(min_cached_at))
            .select((ArtistRow::as_select(), Option::<AreaRow>::as_select()))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(format!("Failed to get cached artist: {}", e)))?;

        Ok(result.map(|(artist_row, area_row)| {
            artist_row.into_artist(area_row.map(Into::into))
        }))
    }

    /// Get an artist by ID (regardless of cache expiry).
    #[allow(dead_code)] // Used in integration tests
    pub async fn get_by_id(pool: &DbPool, id: &str) -> Result<Option<Artist>> {
        let uuid = parse_uuid(id)?;
        let mut conn = get_conn(pool).await?;

        let result: Option<(ArtistRow, Option<AreaRow>)> = artists::table
            .left_join(areas::table)
            .filter(artists::id.eq(uuid))
            .select((ArtistRow::as_select(), Option::<AreaRow>::as_select()))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(format!("Failed to get artist: {}", e)))?;

        Ok(result.map(|(artist_row, area_row)| {
            artist_row.into_artist(area_row.map(Into::into))
        }))
    }

    /// Get multiple artists by their IDs.
    ///
    /// # Errors
    /// Returns an error if the batch size exceeds the maximum allowed (100).
    pub async fn get_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Artist>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        validate_batch_size(ids.len())?;

        let mut conn = get_conn(pool).await?;

        let results: Vec<(ArtistRow, Option<AreaRow>)> = artists::table
            .left_join(areas::table)
            .filter(artists::id.eq_any(ids))
            .select((ArtistRow::as_select(), Option::<AreaRow>::as_select()))
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Database(format!("Failed to get artists by IDs: {}", e)))?;

        Ok(results
            .into_iter()
            .map(|(artist_row, area_row)| artist_row.into_artist(area_row.map(Into::into)))
            .collect())
    }

    /// Upsert an artist (insert or update).
    ///
    /// Area and artist upserts are performed atomically within a transaction.
    pub async fn upsert(pool: &DbPool, artist: &Artist) -> Result<()> {
        let mut conn = get_conn(pool).await?;

        let new_area = artist
            .area
            .as_ref()
            .map(|area| {
                NewAreaRow::try_from(area)
                    .map_err(|e: uuid::Error| AppError::Database(format!("Invalid area UUID: {}", e)))
            })
            .transpose()?;

        let new_artist = NewArtistRow::try_from(artist)
            .map_err(|e| AppError::Database(format!("Invalid artist UUID: {}", e)))?;

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                if let Some(ref new_area) = new_area {
                    diesel::insert_into(areas::table)
                        .values(new_area)
                        .on_conflict(areas::id)
                        .do_update()
                        .set((
                            areas::name.eq(&new_area.name),
                            areas::sort_name.eq(&new_area.sort_name),
                            areas::updated_at.eq(Utc::now()),
                        ))
                        .execute(conn)
                        .await
                        .map_err(|e| AppError::Database(format!("Failed to upsert area: {}", e)))?;
                }

                diesel::insert_into(artists::table)
                    .values(&new_artist)
                    .on_conflict(artists::id)
                    .do_update()
                    .set((
                        artists::name.eq(&new_artist.name),
                        artists::sort_name.eq(&new_artist.sort_name),
                        artists::artist_type.eq(&new_artist.artist_type),
                        artists::country.eq(&new_artist.country),
                        artists::area_id.eq(&new_artist.area_id),
                        artists::disambiguation.eq(&new_artist.disambiguation),
                        artists::life_span.eq(&new_artist.life_span),
                        artists::updated_at.eq(Utc::now()),
                        artists::cached_at.eq(Utc::now()),
                    ))
                    .execute(conn)
                    .await
                    .map_err(|e| AppError::Database(format!("Failed to upsert artist: {}", e)))?;

                Ok(())
            })
        })
        .await
    }

    /// Upsert multiple artists using batch operations.
    pub async fn upsert_many(pool: &DbPool, artists_list: &[Artist]) -> Result<()> {
        if artists_list.is_empty() {
            return Ok(());
        }

        let mut conn = get_conn(pool).await?;

        let new_areas: Vec<NewAreaRow> = artists_list
            .iter()
            .filter_map(|a| a.area.as_ref())
            .filter_map(|area| {
                NewAreaRow::try_from(area).map_err(|e| {
                    tracing::warn!(area_id = %area.id, error = %e, "Skipping area with invalid UUID");
                    e
                }).ok()
            })
            .collect();

        if !new_areas.is_empty() {
            diesel::insert_into(areas::table)
                .values(&new_areas)
                .on_conflict(areas::id)
                .do_update()
                .set((
                    areas::name.eq(diesel::dsl::sql::<diesel::sql_types::Text>("excluded.name")),
                    areas::sort_name.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.sort_name")),
                    areas::updated_at.eq(Utc::now()),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::Database(format!("Failed to batch upsert areas: {}", e)))?;
        }

        let new_artists: Vec<NewArtistRow> = artists_list
            .iter()
            .filter_map(|a| {
                NewArtistRow::try_from(a).map_err(|e| {
                    tracing::warn!(artist_id = %a.id, error = %e, "Skipping artist with invalid UUID");
                    e
                }).ok()
            })
            .collect();

        if new_artists.is_empty() {
            return Ok(());
        }

        diesel::insert_into(artists::table)
            .values(&new_artists)
            .on_conflict(artists::id)
            .do_update()
            .set((
                artists::name.eq(diesel::dsl::sql::<diesel::sql_types::Text>("excluded.name")),
                artists::sort_name.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.sort_name")),
                artists::artist_type.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.artist_type")),
                artists::country.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.country")),
                artists::area_id.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>>("excluded.area_id")),
                artists::disambiguation.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("excluded.disambiguation")),
                artists::life_span.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>>("excluded.life_span")),
                artists::updated_at.eq(Utc::now()),
                artists::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Database(format!("Failed to batch upsert artists: {}", e)))?;

        Ok(())
    }
}
