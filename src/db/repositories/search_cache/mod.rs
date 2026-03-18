mod helpers;
#[macro_use]
mod macros;

use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sha2::{Digest, Sha256};
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

pub fn hash_query(query: &str, limit: i32, offset: i32) -> String {
    let mut hasher = Sha256::new();
    hasher.update(query.as_bytes());
    hasher.update(limit.to_le_bytes());
    hasher.update(offset.to_le_bytes());
    hex::encode(hasher.finalize())
}

pub struct SearchCacheRepository;

impl SearchCacheRepository {
    define_search_cache! {
        entity_name: "artist",
        get_fn: get_artist_search,
        cache_fn: cache_artist_search,
        entity_type: Artist,
        cache_row: ArtistSearchCacheRow,
        table: artist_search_cache,
        id_type: Uuid,
        ids_column: artist_ids,
        extract_ids: |row: ArtistSearchCacheRow| row.artist_ids.into_iter().flatten().collect(),
        get_by_ids: ArtistRepository::get_by_ids,
        entity_id_fn: |a: &Artist| a.id.clone(),
        make_ids: |entities: &[Artist]| entities.iter().filter_map(|a| Uuid::parse_str(&a.id).ok()).collect(),
        upsert_many: ArtistRepository::upsert_many,
        new_row: |qh: String, qt: String, ids: Vec<Uuid>, count: i64| NewArtistSearchCacheRow {
            query_hash: qh, query_text: qt, artist_ids: ids, total_count: count,
        },
    }

    define_search_cache! {
        entity_name: "release",
        get_fn: get_release_search,
        cache_fn: cache_release_search,
        entity_type: Release,
        cache_row: ReleaseSearchCacheRow,
        table: release_search_cache,
        id_type: Uuid,
        ids_column: release_ids,
        extract_ids: |row: ReleaseSearchCacheRow| row.release_ids.into_iter().flatten().collect(),
        get_by_ids: ReleaseRepository::get_by_ids,
        entity_id_fn: |r: &Release| r.id.clone(),
        make_ids: |entities: &[Release]| entities.iter().filter_map(|r| Uuid::parse_str(&r.id).ok()).collect(),
        upsert_many: ReleaseRepository::upsert_many,
        new_row: |qh: String, qt: String, ids: Vec<Uuid>, count: i64| NewReleaseSearchCacheRow {
            query_hash: qh, query_text: qt, release_ids: ids, total_count: count,
        },
    }

    define_search_cache! {
        entity_name: "recording",
        get_fn: get_recording_search,
        cache_fn: cache_recording_search,
        entity_type: Recording,
        cache_row: RecordingSearchCacheRow,
        table: recording_search_cache,
        id_type: Uuid,
        ids_column: recording_ids,
        extract_ids: |row: RecordingSearchCacheRow| row.recording_ids.into_iter().flatten().collect(),
        get_by_ids: RecordingRepository::get_by_ids,
        entity_id_fn: |r: &Recording| r.id.clone(),
        make_ids: |entities: &[Recording]| entities.iter().filter_map(|r| Uuid::parse_str(&r.id).ok()).collect(),
        upsert_many: RecordingRepository::upsert_many,
        new_row: |qh: String, qt: String, ids: Vec<Uuid>, count: i64| NewRecordingSearchCacheRow {
            query_hash: qh, query_text: qt, recording_ids: ids, total_count: count,
        },
    }

    define_search_cache! {
        entity_name: "release_group",
        get_fn: get_release_group_search,
        cache_fn: cache_release_group_search,
        entity_type: ReleaseGroup,
        cache_row: ReleaseGroupSearchCacheRow,
        table: release_group_search_cache,
        id_type: Uuid,
        ids_column: release_group_ids,
        extract_ids: |row: ReleaseGroupSearchCacheRow| row.release_group_ids.into_iter().flatten().collect(),
        get_by_ids: ReleaseGroupRepository::get_by_ids,
        entity_id_fn: |rg: &ReleaseGroup| rg.id.clone(),
        make_ids: |entities: &[ReleaseGroup]| entities.iter().filter_map(|rg| Uuid::parse_str(&rg.id).ok()).collect(),
        upsert_many: ReleaseGroupRepository::upsert_many,
        new_row: |qh: String, qt: String, ids: Vec<Uuid>, count: i64| NewReleaseGroupSearchCacheRow {
            query_hash: qh, query_text: qt, release_group_ids: ids, total_count: count,
        },
    }

    define_search_cache! {
        entity_name: "audiobook",
        get_fn: get_audiobook_search,
        cache_fn: cache_audiobook_search,
        entity_type: Audiobook,
        cache_row: AudiobookSearchCacheRow,
        table: audiobook_search_cache,
        id_type: i64,
        ids_column: audiobook_ids,
        extract_ids: |row: AudiobookSearchCacheRow| row.audiobook_ids.into_iter().flatten().collect(),
        get_by_ids: AudiobookRepository::get_by_ids,
        entity_id_fn: |a: &Audiobook| a.id.to_string(),
        make_ids: |entities: &[Audiobook]| entities.iter().map(|a| a.id).collect(),
        upsert_many: AudiobookRepository::upsert_many,
        new_row: |qh: String, qt: String, ids: Vec<i64>, count: i64| NewAudiobookSearchCacheRow {
            query_hash: qh, query_text: qt, audiobook_ids: ids, total_count: count,
        },
    }

    define_search_cache! {
        entity_name: "podcast",
        get_fn: get_podcast_search,
        cache_fn: cache_podcast_search,
        entity_type: Podcast,
        cache_row: PodcastSearchCacheRow,
        table: podcast_search_cache,
        id_type: i64,
        ids_column: podcast_ids,
        extract_ids: |row: PodcastSearchCacheRow| row.podcast_ids.into_iter().flatten().collect(),
        get_by_ids: PodcastRepository::get_by_ids,
        entity_id_fn: |p: &Podcast| p.id.to_string(),
        make_ids: |entities: &[Podcast]| entities.iter().map(|p| p.id).collect(),
        upsert_many: PodcastRepository::upsert_many,
        new_row: |qh: String, qt: String, ids: Vec<i64>, count: i64| NewPodcastSearchCacheRow {
            query_hash: qh, query_text: qt, podcast_ids: ids, total_count: count,
        },
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
