use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

use crate::db::models::{
    ArtistSearchCacheRow, AudiobookSearchCacheRow, NewArtistSearchCacheRow,
    NewAudiobookSearchCacheRow, NewPodcastSearchCacheRow, NewRecordingSearchCacheRow,
    NewReleaseGroupSearchCacheRow, NewReleaseSearchCacheRow, PodcastSearchCacheRow,
    RecordingSearchCacheRow, ReleaseGroupSearchCacheRow, ReleaseSearchCacheRow,
};
use crate::db::repositories::{
    ArtistRepository, AudiobookRepository, PodcastRepository, RecordingRepository,
    ReleaseGroupRepository, ReleaseRepository,
};
use crate::db::schema::{
    artist_search_cache, audiobook_search_cache, podcast_search_cache, recording_search_cache,
    release_group_search_cache, release_search_cache,
};
use crate::db::{DbPool, db_error, get_conn, min_cached_at};
use crate::audiobook::Audiobook;
use crate::error::Result;
use crate::musicbrainz::{Artist, Recording, Release, ReleaseGroup};
use crate::podcast::Podcast;

/// Returns `false` if some IDs are missing (stale cache).
fn all_ids_resolved(entity: &str, expected: usize, actual: usize) -> bool {
    if actual < expected {
        tracing::warn!(
            entity = entity,
            expected = expected,
            actual = actual,
            "Some cached {} IDs did not resolve — treating as cache miss",
            entity,
        );
        false
    } else {
        true
    }
}

pub fn hash_query(query: &str, limit: i32, offset: i32) -> String {
    let mut hasher = Sha256::new();
    hasher.update(query.as_bytes());
    hasher.update(limit.to_le_bytes());
    hasher.update(offset.to_le_bytes());
    hex::encode(hasher.finalize())
}

pub struct SearchCacheRepository;

impl SearchCacheRepository {
    // ==================== Artist Search ====================

    pub async fn get_artist_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Vec<Artist>>> {
        let query_hash = hash_query(query, limit, offset);
        let min_cached_at = min_cached_at(cache_ttl_seconds)?;
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
                let by_id: HashMap<String, Artist> =
                    artists.into_iter().map(|a| (a.id.clone(), a)).collect();
                let ordered: Vec<Artist> = artist_ids
                    .iter()
                    .filter_map(|id| by_id.get(&id.to_string()).cloned())
                    .collect();
                if !all_ids_resolved("artist", artist_ids.len(), ordered.len()) {
                    return Ok(None);
                }
                Ok(Some(ordered))
            }
            None => Ok(None),
        }
    }

    pub async fn cache_artist_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        artists: &[Artist],
    ) -> Result<()> {
        ArtistRepository::upsert_many(pool, artists).await?;

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

    pub async fn get_release_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Vec<Release>>> {
        let query_hash = hash_query(query, limit, offset);
        let min_cached_at = min_cached_at(cache_ttl_seconds)?;
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
                let by_id: HashMap<String, Release> =
                    releases.into_iter().map(|r| (r.id.clone(), r)).collect();
                let ordered: Vec<Release> = release_ids
                    .iter()
                    .filter_map(|id| by_id.get(&id.to_string()).cloned())
                    .collect();
                if !all_ids_resolved("release", release_ids.len(), ordered.len()) {
                    return Ok(None);
                }
                Ok(Some(ordered))
            }
            None => Ok(None),
        }
    }

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

    pub async fn get_recording_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Vec<Recording>>> {
        let query_hash = hash_query(query, limit, offset);
        let min_cached_at = min_cached_at(cache_ttl_seconds)?;
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
                let by_id: HashMap<String, Recording> =
                    recordings.into_iter().map(|r| (r.id.clone(), r)).collect();
                let ordered: Vec<Recording> = recording_ids
                    .iter()
                    .filter_map(|id| by_id.get(&id.to_string()).cloned())
                    .collect();
                if !all_ids_resolved("recording", recording_ids.len(), ordered.len()) {
                    return Ok(None);
                }
                Ok(Some(ordered))
            }
            None => Ok(None),
        }
    }

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

    pub async fn get_release_group_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Vec<ReleaseGroup>>> {
        let query_hash = hash_query(query, limit, offset);
        let min_cached_at = min_cached_at(cache_ttl_seconds)?;
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
                let by_id: HashMap<String, ReleaseGroup> = release_groups
                    .into_iter()
                    .map(|rg| (rg.id.clone(), rg))
                    .collect();
                let ordered: Vec<ReleaseGroup> = release_group_ids
                    .iter()
                    .filter_map(|id| by_id.get(&id.to_string()).cloned())
                    .collect();
                if !all_ids_resolved("release_group", release_group_ids.len(), ordered.len()) {
                    return Ok(None);
                }
                Ok(Some(ordered))
            }
            None => Ok(None),
        }
    }

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

    // ==================== Audiobook Search ====================

    pub async fn get_audiobook_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Vec<Audiobook>>> {
        let query_hash = hash_query(query, limit, offset);
        let min_cached_at = min_cached_at(cache_ttl_seconds)?;
        let mut conn = get_conn(pool).await?;

        let cache_row: Option<AudiobookSearchCacheRow> = audiobook_search_cache::table
            .filter(audiobook_search_cache::query_hash.eq(&query_hash))
            .filter(audiobook_search_cache::cached_at.gt(min_cached_at))
            .select(AudiobookSearchCacheRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get audiobook search cache"))?;

        match cache_row {
            Some(row) => {
                let audiobook_ids: Vec<i64> = row.audiobook_ids.into_iter().flatten().collect();
                let audiobooks = AudiobookRepository::get_by_ids(pool, &audiobook_ids).await?;
                let by_id: HashMap<i64, Audiobook> =
                    audiobooks.into_iter().map(|a| (a.id, a)).collect();
                let ordered: Vec<Audiobook> = audiobook_ids
                    .iter()
                    .filter_map(|id| by_id.get(id).cloned())
                    .collect();
                if !all_ids_resolved("audiobook", audiobook_ids.len(), ordered.len()) {
                    return Ok(None);
                }
                Ok(Some(ordered))
            }
            None => Ok(None),
        }
    }

    pub async fn cache_audiobook_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        audiobooks: &[Audiobook],
    ) -> Result<()> {
        AudiobookRepository::upsert_many(pool, audiobooks).await?;

        let audiobook_ids: Vec<i64> = audiobooks.iter().map(|a| a.id).collect();

        let query_hash = hash_query(query, limit, offset);

        let new_cache = NewAudiobookSearchCacheRow {
            query_hash: query_hash.clone(),
            query_text: query.to_string(),
            audiobook_ids,
            total_count: audiobooks.len() as i32,
        };

        let mut conn = get_conn(pool).await?;

        diesel::insert_into(audiobook_search_cache::table)
            .values(&new_cache)
            .on_conflict(audiobook_search_cache::query_hash)
            .do_update()
            .set((
                audiobook_search_cache::audiobook_ids.eq(&new_cache.audiobook_ids),
                audiobook_search_cache::total_count.eq(&new_cache.total_count),
                audiobook_search_cache::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to cache audiobook search"))?;

        Ok(())
    }

    // ==================== Podcast Search ====================

    pub async fn get_podcast_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Vec<Podcast>>> {
        let query_hash = hash_query(query, limit, offset);
        let min_cached_at = min_cached_at(cache_ttl_seconds)?;
        let mut conn = get_conn(pool).await?;

        let cache_row: Option<PodcastSearchCacheRow> = podcast_search_cache::table
            .filter(podcast_search_cache::query_hash.eq(&query_hash))
            .filter(podcast_search_cache::cached_at.gt(min_cached_at))
            .select(PodcastSearchCacheRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get podcast search cache"))?;

        match cache_row {
            Some(row) => {
                let podcast_ids: Vec<i64> = row.podcast_ids.into_iter().flatten().collect();
                let podcasts = PodcastRepository::get_by_ids(pool, &podcast_ids).await?;
                let by_id: HashMap<i64, Podcast> =
                    podcasts.into_iter().map(|p| (p.id, p)).collect();
                let ordered: Vec<Podcast> = podcast_ids
                    .iter()
                    .filter_map(|id| by_id.get(id).cloned())
                    .collect();
                if !all_ids_resolved("podcast", podcast_ids.len(), ordered.len()) {
                    return Ok(None);
                }
                Ok(Some(ordered))
            }
            None => Ok(None),
        }
    }

    pub async fn cache_podcast_search(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        podcasts: &[Podcast],
    ) -> Result<()> {
        PodcastRepository::upsert_many(pool, podcasts).await?;

        let podcast_ids: Vec<i64> = podcasts.iter().map(|p| p.id).collect();

        let query_hash = hash_query(query, limit, offset);

        let new_cache = NewPodcastSearchCacheRow {
            query_hash: query_hash.clone(),
            query_text: query.to_string(),
            podcast_ids,
            total_count: podcasts.len() as i32,
        };

        let mut conn = get_conn(pool).await?;

        diesel::insert_into(podcast_search_cache::table)
            .values(&new_cache)
            .on_conflict(podcast_search_cache::query_hash)
            .do_update()
            .set((
                podcast_search_cache::podcast_ids.eq(&new_cache.podcast_ids),
                podcast_search_cache::total_count.eq(&new_cache.total_count),
                podcast_search_cache::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to cache podcast search"))?;

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
