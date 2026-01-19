//! Artist repository for database operations.

use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::models::{AreaRow, ArtistRow, NewAreaRow, NewArtistRow};
use crate::db::schema::{areas, artists};
use crate::db::DbPool;
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
        let uuid = Uuid::parse_str(id).map_err(|e| AppError::Database(e.to_string()))?;
        let min_cached_at = Utc::now() - Duration::seconds(cache_ttl_seconds);

        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let result: Option<(ArtistRow, Option<AreaRow>)> = artists::table
            .left_join(areas::table)
            .filter(artists::id.eq(uuid))
            .filter(artists::cached_at.gt(min_cached_at))
            .select((ArtistRow::as_select(), Option::<AreaRow>::as_select()))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(|(artist_row, area_row)| {
            artist_row.into_artist(area_row.map(Into::into))
        }))
    }

    /// Get an artist by ID (regardless of cache expiry).
    #[allow(dead_code)] // Used in integration tests
    pub async fn get_by_id(pool: &DbPool, id: &str) -> Result<Option<Artist>> {
        let uuid = Uuid::parse_str(id).map_err(|e| AppError::Database(e.to_string()))?;

        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let result: Option<(ArtistRow, Option<AreaRow>)> = artists::table
            .left_join(areas::table)
            .filter(artists::id.eq(uuid))
            .select((ArtistRow::as_select(), Option::<AreaRow>::as_select()))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(|(artist_row, area_row)| {
            artist_row.into_artist(area_row.map(Into::into))
        }))
    }

    /// Get multiple artists by their IDs.
    pub async fn get_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Artist>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        let results: Vec<(ArtistRow, Option<AreaRow>)> = artists::table
            .left_join(areas::table)
            .filter(artists::id.eq_any(ids))
            .select((ArtistRow::as_select(), Option::<AreaRow>::as_select()))
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(results
            .into_iter()
            .map(|(artist_row, area_row)| artist_row.into_artist(area_row.map(Into::into)))
            .collect())
    }

    /// Upsert an artist (insert or update).
    pub async fn upsert(pool: &DbPool, artist: &Artist) -> Result<()> {
        let mut conn = pool.get().await.map_err(|e| AppError::Database(e.to_string()))?;

        // First, upsert the area if present
        if let Some(ref area) = artist.area {
            let new_area =
                NewAreaRow::try_from(area).map_err(|e: uuid::Error| AppError::Database(e.to_string()))?;

            diesel::insert_into(areas::table)
                .values(&new_area)
                .on_conflict(areas::id)
                .do_update()
                .set((
                    areas::name.eq(&new_area.name),
                    areas::sort_name.eq(&new_area.sort_name),
                    areas::updated_at.eq(Utc::now()),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e: diesel::result::Error| AppError::Database(e.to_string()))?;
        }

        // Then upsert the artist
        let new_artist =
            NewArtistRow::try_from(artist).map_err(|e| AppError::Database(e.to_string()))?;

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
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Upsert multiple artists.
    pub async fn upsert_many(pool: &DbPool, artists_list: &[Artist]) -> Result<()> {
        for artist in artists_list {
            Self::upsert(pool, artist).await?;
        }
        Ok(())
    }
}
