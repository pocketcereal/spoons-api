//! Search cache repository for database operations.

use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::db::models::{
    ArtistSearchCacheRow, NewArtistSearchCacheRow, NewRecordingSearchCacheRow,
    NewReleaseGroupSearchCacheRow, NewReleaseSearchCacheRow, RecordingSearchCacheRow,
    ReleaseGroupSearchCacheRow, ReleaseSearchCacheRow,
};
use crate::db::repositories::{
    ArtistRepository, RecordingRepository, ReleaseGroupRepository, ReleaseRepository,
};
use crate::db::schema::{
    artist_search_cache, recording_search_cache, release_group_search_cache, release_search_cache,
};
use crate::db::{db_error, get_conn, DbPool};
use crate::error::Result;
use crate::musicbrainz::{Artist, Recording, Release, ReleaseGroup};

/// Generate a hash for a search query with pagination.
pub fn hash_query(query: &str, limit: i32, offset: i32) -> String {
    let mut hasher = Sha256::new();
    hasher.update(query.as_bytes());
    hasher.update(limit.to_le_bytes());
    hasher.update(offset.to_le_bytes());
    hex::encode(hasher.finalize())
}

/// Repository for search cache operations.
pub struct SearchCacheRepository;

impl SearchCacheRepository {
    // ==================== Artist Search ====================

    /// Get cached artist search results if not expired.
    pub async fn get_artist_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Vec<Artist>>> {
        let query_hash = hash_query(query, limit, offset);
        let min_cached_at = Utc::now() - Duration::seconds(cache_ttl_seconds);
        let mut conn = get_conn(pool).await?;

        let cache_row: Option<ArtistSearchCacheRow> = artist_search_cache::table
            .filter(artist_search_cache::query_hash.eq(&query_hash))
            .filter(artist_search_cache::cached_at.gt(min_cached_at))
            .select(ArtistSearchCacheRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get artist search cache"))?;

        match cache_row {
            Some(row) => {
                let artist_ids: Vec<Uuid> = row.artist_ids.into_iter().flatten().collect();
                let artists = ArtistRepository::get_by_ids(pool, &artist_ids).await?;
                Ok(Some(artists))
            }
            None => Ok(None),
        }
    }

    /// Cache artist search results.
    pub async fn cache_artist_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        artists: &[Artist],
    ) -> Result<()> {
        // First upsert all artists
        ArtistRepository::upsert_many(pool, artists).await?;

        // Then cache the search result
        let artist_ids: Vec<Uuid> = artists
            .iter()
            .filter_map(|a| Uuid::parse_str(&a.id).ok())
            .collect();

        let query_hash = hash_query(query, limit, offset);

        let new_cache = NewArtistSearchCacheRow {
            query_hash: query_hash.clone(),
            query_text: query.to_string(),
            artist_ids,
            total_count: artists.len() as i64,
        };

        let mut conn = get_conn(pool).await?;

        diesel::insert_into(artist_search_cache::table)
            .values(&new_cache)
            .on_conflict(artist_search_cache::query_hash)
            .do_update()
            .set((
                artist_search_cache::artist_ids.eq(&new_cache.artist_ids),
                artist_search_cache::total_count.eq(&new_cache.total_count),
                artist_search_cache::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to cache artist search"))?;

        Ok(())
    }

    // ==================== Release Search ====================

    /// Get cached release search results if not expired.
    pub async fn get_release_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Vec<Release>>> {
        let query_hash = hash_query(query, limit, offset);
        let min_cached_at = Utc::now() - Duration::seconds(cache_ttl_seconds);
        let mut conn = get_conn(pool).await?;

        let cache_row: Option<ReleaseSearchCacheRow> = release_search_cache::table
            .filter(release_search_cache::query_hash.eq(&query_hash))
            .filter(release_search_cache::cached_at.gt(min_cached_at))
            .select(ReleaseSearchCacheRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get release search cache"))?;

        match cache_row {
            Some(row) => {
                let release_ids: Vec<Uuid> = row.release_ids.into_iter().flatten().collect();
                let releases = ReleaseRepository::get_by_ids(pool, &release_ids).await?;
                Ok(Some(releases))
            }
            None => Ok(None),
        }
    }

    /// Cache release search results.
    pub async fn cache_release_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        releases: &[Release],
    ) -> Result<()> {
        ReleaseRepository::upsert_many(pool, releases).await?;

        let release_ids: Vec<Uuid> = releases
            .iter()
            .filter_map(|r| Uuid::parse_str(&r.id).ok())
            .collect();

        let query_hash = hash_query(query, limit, offset);

        let new_cache = NewReleaseSearchCacheRow {
            query_hash: query_hash.clone(),
            query_text: query.to_string(),
            release_ids,
            total_count: releases.len() as i64,
        };

        let mut conn = get_conn(pool).await?;

        diesel::insert_into(release_search_cache::table)
            .values(&new_cache)
            .on_conflict(release_search_cache::query_hash)
            .do_update()
            .set((
                release_search_cache::release_ids.eq(&new_cache.release_ids),
                release_search_cache::total_count.eq(&new_cache.total_count),
                release_search_cache::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to cache release search"))?;

        Ok(())
    }

    // ==================== Recording Search ====================

    /// Get cached recording search results if not expired.
    pub async fn get_recording_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Vec<Recording>>> {
        let query_hash = hash_query(query, limit, offset);
        let min_cached_at = Utc::now() - Duration::seconds(cache_ttl_seconds);
        let mut conn = get_conn(pool).await?;

        let cache_row: Option<RecordingSearchCacheRow> = recording_search_cache::table
            .filter(recording_search_cache::query_hash.eq(&query_hash))
            .filter(recording_search_cache::cached_at.gt(min_cached_at))
            .select(RecordingSearchCacheRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get recording search cache"))?;

        match cache_row {
            Some(row) => {
                let recording_ids: Vec<Uuid> = row.recording_ids.into_iter().flatten().collect();
                let recordings = RecordingRepository::get_by_ids(pool, &recording_ids).await?;
                Ok(Some(recordings))
            }
            None => Ok(None),
        }
    }

    /// Cache recording search results.
    pub async fn cache_recording_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        recordings: &[Recording],
    ) -> Result<()> {
        RecordingRepository::upsert_many(pool, recordings).await?;

        let recording_ids: Vec<Uuid> = recordings
            .iter()
            .filter_map(|r| Uuid::parse_str(&r.id).ok())
            .collect();

        let query_hash = hash_query(query, limit, offset);

        let new_cache = NewRecordingSearchCacheRow {
            query_hash: query_hash.clone(),
            query_text: query.to_string(),
            recording_ids,
            total_count: recordings.len() as i64,
        };

        let mut conn = get_conn(pool).await?;

        diesel::insert_into(recording_search_cache::table)
            .values(&new_cache)
            .on_conflict(recording_search_cache::query_hash)
            .do_update()
            .set((
                recording_search_cache::recording_ids.eq(&new_cache.recording_ids),
                recording_search_cache::total_count.eq(&new_cache.total_count),
                recording_search_cache::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to cache recording search"))?;

        Ok(())
    }

    // ==================== Release Group Search ====================

    /// Get cached release group search results if not expired.
    pub async fn get_release_group_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Vec<ReleaseGroup>>> {
        let query_hash = hash_query(query, limit, offset);
        let min_cached_at = Utc::now() - Duration::seconds(cache_ttl_seconds);
        let mut conn = get_conn(pool).await?;

        let cache_row: Option<ReleaseGroupSearchCacheRow> = release_group_search_cache::table
            .filter(release_group_search_cache::query_hash.eq(&query_hash))
            .filter(release_group_search_cache::cached_at.gt(min_cached_at))
            .select(ReleaseGroupSearchCacheRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get release group search cache"))?;

        match cache_row {
            Some(row) => {
                let release_group_ids: Vec<Uuid> =
                    row.release_group_ids.into_iter().flatten().collect();
                let release_groups =
                    ReleaseGroupRepository::get_by_ids(pool, &release_group_ids).await?;
                Ok(Some(release_groups))
            }
            None => Ok(None),
        }
    }

    /// Cache release group search results.
    pub async fn cache_release_group_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        release_groups: &[ReleaseGroup],
    ) -> Result<()> {
        ReleaseGroupRepository::upsert_many(pool, release_groups).await?;

        let release_group_ids: Vec<Uuid> = release_groups
            .iter()
            .filter_map(|rg| Uuid::parse_str(&rg.id).ok())
            .collect();

        let query_hash = hash_query(query, limit, offset);

        let new_cache = NewReleaseGroupSearchCacheRow {
            query_hash: query_hash.clone(),
            query_text: query.to_string(),
            release_group_ids,
            total_count: release_groups.len() as i64,
        };

        let mut conn = get_conn(pool).await?;

        diesel::insert_into(release_group_search_cache::table)
            .values(&new_cache)
            .on_conflict(release_group_search_cache::query_hash)
            .do_update()
            .set((
                release_group_search_cache::release_group_ids.eq(&new_cache.release_group_ids),
                release_group_search_cache::total_count.eq(&new_cache.total_count),
                release_group_search_cache::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to cache release group search"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_query() {
        let hash1 = hash_query("nirvana", 25, 0);
        let hash2 = hash_query("nirvana", 25, 0);
        let hash3 = hash_query("nirvana", 25, 10);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
