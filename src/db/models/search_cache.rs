//! Search cache database models.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::schema::{
    artist_search_cache, audiobook_search_cache, podcast_search_cache, recording_search_cache,
    release_group_search_cache, release_search_cache,
};

/// Database row for artist_search_cache table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = artist_search_cache)]
#[diesel(primary_key(query_hash))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ArtistSearchCacheRow {
    pub query_hash: String,
    pub query_text: String,
    pub artist_ids: Vec<Option<Uuid>>,
    pub total_count: i64,
    pub cached_at: DateTime<Utc>,
}

/// Insertable row for artist_search_cache table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = artist_search_cache)]
pub struct NewArtistSearchCacheRow {
    pub query_hash: String,
    pub query_text: String,
    pub artist_ids: Vec<Uuid>,
    pub total_count: i64,
}

/// Database row for release_search_cache table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = release_search_cache)]
#[diesel(primary_key(query_hash))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ReleaseSearchCacheRow {
    pub query_hash: String,
    pub query_text: String,
    pub release_ids: Vec<Option<Uuid>>,
    pub total_count: i64,
    pub cached_at: DateTime<Utc>,
}

/// Insertable row for release_search_cache table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = release_search_cache)]
pub struct NewReleaseSearchCacheRow {
    pub query_hash: String,
    pub query_text: String,
    pub release_ids: Vec<Uuid>,
    pub total_count: i64,
}

/// Database row for recording_search_cache table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = recording_search_cache)]
#[diesel(primary_key(query_hash))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RecordingSearchCacheRow {
    pub query_hash: String,
    pub query_text: String,
    pub recording_ids: Vec<Option<Uuid>>,
    pub total_count: i64,
    pub cached_at: DateTime<Utc>,
}

/// Insertable row for recording_search_cache table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = recording_search_cache)]
pub struct NewRecordingSearchCacheRow {
    pub query_hash: String,
    pub query_text: String,
    pub recording_ids: Vec<Uuid>,
    pub total_count: i64,
}

/// Database row for release_group_search_cache table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = release_group_search_cache)]
#[diesel(primary_key(query_hash))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ReleaseGroupSearchCacheRow {
    pub query_hash: String,
    pub query_text: String,
    pub release_group_ids: Vec<Option<Uuid>>,
    pub total_count: i64,
    pub cached_at: DateTime<Utc>,
}

/// Insertable row for release_group_search_cache table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = release_group_search_cache)]
pub struct NewReleaseGroupSearchCacheRow {
    pub query_hash: String,
    pub query_text: String,
    pub release_group_ids: Vec<Uuid>,
    pub total_count: i64,
}

/// Database row for audiobook_search_cache table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = audiobook_search_cache)]
#[diesel(primary_key(query_hash))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AudiobookSearchCacheRow {
    pub query_hash: String,
    pub query_text: String,
    pub audiobook_ids: Vec<Option<i64>>,
    pub total_count: i32,
    pub cached_at: DateTime<Utc>,
}

/// Insertable row for audiobook_search_cache table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = audiobook_search_cache)]
pub struct NewAudiobookSearchCacheRow {
    pub query_hash: String,
    pub query_text: String,
    pub audiobook_ids: Vec<i64>,
    pub total_count: i32,
}

/// Database row for podcast_search_cache table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = podcast_search_cache)]
#[diesel(primary_key(query_hash))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PodcastSearchCacheRow {
    pub query_hash: String,
    pub query_text: String,
    pub podcast_ids: Vec<Option<i64>>,
    pub total_count: i32,
    pub cached_at: DateTime<Utc>,
}

/// Insertable row for podcast_search_cache table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = podcast_search_cache)]
pub struct NewPodcastSearchCacheRow {
    pub query_hash: String,
    pub query_text: String,
    pub podcast_ids: Vec<i64>,
    pub total_count: i32,
}
