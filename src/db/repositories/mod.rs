//! Database repositories for music cache operations.

mod artist;
mod recording;
mod release;
mod release_group;
mod search_cache;

pub use artist::ArtistRepository;
pub use recording::RecordingRepository;
pub use release::ReleaseRepository;
pub use release_group::ReleaseGroupRepository;
pub use search_cache::SearchCacheRepository;

use crate::db::DbPool;
use crate::error::Result;
use crate::musicbrainz::{Artist, MusicBrainzClient, Recording, Release, ReleaseGroup};

/// Facade for all music cache operations with cache-first pattern.
pub struct MusicRepository;

impl MusicRepository {
    /// Get an artist by ID, checking cache first.
    pub async fn get_artist(
        pool: &DbPool,
        client: &MusicBrainzClient,
        id: &str,
        cache_ttl_seconds: i64,
    ) -> Result<Artist> {
        // Check cache first
        if let Some(artist) = ArtistRepository::get_cached(pool, id, cache_ttl_seconds).await? {
            tracing::debug!(id, "Artist cache hit");
            return Ok(artist);
        }

        // Cache miss - fetch from API
        tracing::debug!(id, "Artist cache miss, fetching from API");
        let artist = client.get_artist(id).await?;

        // Store in cache (fire and forget)
        let pool_clone = pool.clone();
        let artist_clone = artist.clone();
        tokio::spawn(async move {
            if let Err(e) = ArtistRepository::upsert(&pool_clone, &artist_clone).await {
                tracing::warn!(error = %e, "Failed to cache artist");
            }
        });

        Ok(artist)
    }

    /// Search artists, checking cache first.
    pub async fn search_artists(
        pool: &DbPool,
        client: &MusicBrainzClient,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Vec<Artist>> {
        // Check search cache first
        if let Some(artists) =
            SearchCacheRepository::get_artist_search(pool, query, limit, offset, cache_ttl_seconds)
                .await?
        {
            tracing::debug!(query, "Artist search cache hit");
            return Ok(artists);
        }

        // Cache miss - fetch from API
        tracing::debug!(query, "Artist search cache miss, fetching from API");
        let artists = client.search_artists(query, limit, offset).await?;

        // Store in cache (fire and forget)
        let pool_clone = pool.clone();
        let query_owned = query.to_string();
        let artists_clone = artists.clone();
        tokio::spawn(async move {
            if let Err(e) =
                SearchCacheRepository::cache_artist_search(&pool_clone, &query_owned, &artists_clone)
                    .await
            {
                tracing::warn!(error = %e, "Failed to cache artist search");
            }
        });

        Ok(artists)
    }

    /// Get a release by ID, checking cache first.
    pub async fn get_release(
        pool: &DbPool,
        client: &MusicBrainzClient,
        id: &str,
        cache_ttl_seconds: i64,
    ) -> Result<Release> {
        if let Some(release) = ReleaseRepository::get_cached(pool, id, cache_ttl_seconds).await? {
            tracing::debug!(id, "Release cache hit");
            return Ok(release);
        }

        tracing::debug!(id, "Release cache miss, fetching from API");
        let release = client.get_release(id).await?;

        let pool_clone = pool.clone();
        let release_clone = release.clone();
        tokio::spawn(async move {
            if let Err(e) = ReleaseRepository::upsert(&pool_clone, &release_clone).await {
                tracing::warn!(error = %e, "Failed to cache release");
            }
        });

        Ok(release)
    }

    /// Search releases, checking cache first.
    pub async fn search_releases(
        pool: &DbPool,
        client: &MusicBrainzClient,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Vec<Release>> {
        if let Some(releases) =
            SearchCacheRepository::get_release_search(pool, query, limit, offset, cache_ttl_seconds)
                .await?
        {
            tracing::debug!(query, "Release search cache hit");
            return Ok(releases);
        }

        tracing::debug!(query, "Release search cache miss, fetching from API");
        let releases = client.search_releases(query, limit, offset).await?;

        let pool_clone = pool.clone();
        let query_owned = query.to_string();
        let releases_clone = releases.clone();
        tokio::spawn(async move {
            if let Err(e) =
                SearchCacheRepository::cache_release_search(&pool_clone, &query_owned, &releases_clone)
                    .await
            {
                tracing::warn!(error = %e, "Failed to cache release search");
            }
        });

        Ok(releases)
    }

    /// Get a recording by ID, checking cache first.
    pub async fn get_recording(
        pool: &DbPool,
        client: &MusicBrainzClient,
        id: &str,
        cache_ttl_seconds: i64,
    ) -> Result<Recording> {
        if let Some(recording) =
            RecordingRepository::get_cached(pool, id, cache_ttl_seconds).await?
        {
            tracing::debug!(id, "Recording cache hit");
            return Ok(recording);
        }

        tracing::debug!(id, "Recording cache miss, fetching from API");
        let recording = client.get_recording(id).await?;

        let pool_clone = pool.clone();
        let recording_clone = recording.clone();
        tokio::spawn(async move {
            if let Err(e) = RecordingRepository::upsert(&pool_clone, &recording_clone).await {
                tracing::warn!(error = %e, "Failed to cache recording");
            }
        });

        Ok(recording)
    }

    /// Search recordings, checking cache first.
    pub async fn search_recordings(
        pool: &DbPool,
        client: &MusicBrainzClient,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Vec<Recording>> {
        if let Some(recordings) = SearchCacheRepository::get_recording_search(
            pool,
            query,
            limit,
            offset,
            cache_ttl_seconds,
        )
        .await?
        {
            tracing::debug!(query, "Recording search cache hit");
            return Ok(recordings);
        }

        tracing::debug!(query, "Recording search cache miss, fetching from API");
        let recordings = client.search_recordings(query, limit, offset).await?;

        let pool_clone = pool.clone();
        let query_owned = query.to_string();
        let recordings_clone = recordings.clone();
        tokio::spawn(async move {
            if let Err(e) = SearchCacheRepository::cache_recording_search(
                &pool_clone,
                &query_owned,
                &recordings_clone,
            )
            .await
            {
                tracing::warn!(error = %e, "Failed to cache recording search");
            }
        });

        Ok(recordings)
    }

    /// Get a release group by ID, checking cache first.
    pub async fn get_release_group(
        pool: &DbPool,
        client: &MusicBrainzClient,
        id: &str,
        cache_ttl_seconds: i64,
    ) -> Result<ReleaseGroup> {
        if let Some(release_group) =
            ReleaseGroupRepository::get_cached(pool, id, cache_ttl_seconds).await?
        {
            tracing::debug!(id, "Release group cache hit");
            return Ok(release_group);
        }

        tracing::debug!(id, "Release group cache miss, fetching from API");
        let release_group = client.get_release_group(id).await?;

        let pool_clone = pool.clone();
        let release_group_clone = release_group.clone();
        tokio::spawn(async move {
            if let Err(e) = ReleaseGroupRepository::upsert(&pool_clone, &release_group_clone).await {
                tracing::warn!(error = %e, "Failed to cache release group");
            }
        });

        Ok(release_group)
    }

    /// Search release groups, checking cache first.
    pub async fn search_release_groups(
        pool: &DbPool,
        client: &MusicBrainzClient,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Vec<ReleaseGroup>> {
        if let Some(release_groups) = SearchCacheRepository::get_release_group_search(
            pool,
            query,
            limit,
            offset,
            cache_ttl_seconds,
        )
        .await?
        {
            tracing::debug!(query, "Release group search cache hit");
            return Ok(release_groups);
        }

        tracing::debug!(query, "Release group search cache miss, fetching from API");
        let release_groups = client.search_release_groups(query, limit, offset).await?;

        let pool_clone = pool.clone();
        let query_owned = query.to_string();
        let release_groups_clone = release_groups.clone();
        tokio::spawn(async move {
            if let Err(e) = SearchCacheRepository::cache_release_group_search(
                &pool_clone,
                &query_owned,
                &release_groups_clone,
            )
            .await
            {
                tracing::warn!(error = %e, "Failed to cache release group search");
            }
        });

        Ok(release_groups)
    }
}
