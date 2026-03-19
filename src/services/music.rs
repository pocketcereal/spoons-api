//! Music service implementing cache-first patterns over MusicBrainz.

use crate::cache::cached::{CacheTtlSeconds, cached_get, cached_search};
use crate::db::repositories::{
    ArtistRepository, RecordingRepository, ReleaseGroupRepository, ReleaseRepository,
    SearchCacheRepository,
};
use crate::db::{DbPool, spawn_cache_task};
use crate::error::Result;
use crate::musicbrainz::{Artist, MusicBrainzClient, Recording, Release, ReleaseGroup};

#[derive(Clone)]
pub struct MusicService {
    pool: DbPool,
    mb_client: MusicBrainzClient,
    cache_ttl: CacheTtlSeconds,
}

impl MusicService {
    pub fn new(
        pool: DbPool,
        mb_client: MusicBrainzClient,
        cache_ttl: CacheTtlSeconds,
    ) -> Self {
        Self {
            pool,
            mb_client,
            cache_ttl,
        }
    }

    pub fn mb_client(&self) -> &MusicBrainzClient {
        &self.mb_client
    }

    // === Artists ===

    pub async fn get_artist(&self, id: &str) -> Result<Artist> {
        let pool = self.pool.clone();
        cached_get(
            ArtistRepository::get_cached(&self.pool, id, self.cache_ttl),
            self.mb_client.get_artist(id),
            move |artist: &Artist| {
                let artist = artist.clone();
                spawn_cache_task("artist", move || async move {
                    ArtistRepository::upsert(&pool, &artist).await
                });
            },
            "artist",
            id,
        )
        .await
    }

    pub async fn search_artists(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Artist>> {
        let pool = self.pool.clone();
        let query_owned = query.to_string();
        cached_search(
            SearchCacheRepository::get_artist_search(
                &self.pool,
                query,
                limit,
                offset,
                self.cache_ttl,
            ),
            self.mb_client.search_artists(query, limit, offset),
            move |artists: &[Artist]| {
                let artists = artists.to_vec();
                spawn_cache_task("artist search", move || async move {
                    SearchCacheRepository::cache_artist_search(
                        &pool,
                        &query_owned,
                        limit,
                        offset,
                        &artists,
                    )
                    .await
                });
            },
            "artist",
            query,
        )
        .await
    }

    // === Releases ===

    pub async fn get_release(&self, id: &str) -> Result<Release> {
        let pool = self.pool.clone();
        cached_get(
            ReleaseRepository::get_cached(&self.pool, id, self.cache_ttl),
            self.mb_client.get_release(id),
            move |release: &Release| {
                let release = release.clone();
                spawn_cache_task("release", move || async move {
                    ReleaseRepository::upsert(&pool, &release).await
                });
            },
            "release",
            id,
        )
        .await
    }

    pub async fn search_releases(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Release>> {
        let pool = self.pool.clone();
        let query_owned = query.to_string();
        cached_search(
            SearchCacheRepository::get_release_search(
                &self.pool,
                query,
                limit,
                offset,
                self.cache_ttl,
            ),
            self.mb_client.search_releases(query, limit, offset),
            move |releases: &[Release]| {
                let releases = releases.to_vec();
                spawn_cache_task("release search", move || async move {
                    SearchCacheRepository::cache_release_search(
                        &pool,
                        &query_owned,
                        limit,
                        offset,
                        &releases,
                    )
                    .await
                });
            },
            "release",
            query,
        )
        .await
    }

    // === Recordings ===

    pub async fn get_recording(&self, id: &str) -> Result<Recording> {
        let pool = self.pool.clone();
        cached_get(
            RecordingRepository::get_cached(&self.pool, id, self.cache_ttl),
            self.mb_client.get_recording(id),
            move |recording: &Recording| {
                let recording = recording.clone();
                spawn_cache_task("recording", move || async move {
                    RecordingRepository::upsert(&pool, &recording).await
                });
            },
            "recording",
            id,
        )
        .await
    }

    pub async fn search_recordings(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Recording>> {
        let pool = self.pool.clone();
        let query_owned = query.to_string();
        cached_search(
            SearchCacheRepository::get_recording_search(
                &self.pool,
                query,
                limit,
                offset,
                self.cache_ttl,
            ),
            self.mb_client.search_recordings(query, limit, offset),
            move |recordings: &[Recording]| {
                let recordings = recordings.to_vec();
                spawn_cache_task("recording search", move || async move {
                    SearchCacheRepository::cache_recording_search(
                        &pool,
                        &query_owned,
                        limit,
                        offset,
                        &recordings,
                    )
                    .await
                });
            },
            "recording",
            query,
        )
        .await
    }

    // === Release Groups ===

    pub async fn get_release_group(&self, id: &str) -> Result<ReleaseGroup> {
        let pool = self.pool.clone();
        cached_get(
            ReleaseGroupRepository::get_cached(&self.pool, id, self.cache_ttl),
            self.mb_client.get_release_group(id),
            move |rg: &ReleaseGroup| {
                let rg = rg.clone();
                spawn_cache_task("release group", move || async move {
                    ReleaseGroupRepository::upsert(&pool, &rg).await
                });
            },
            "release_group",
            id,
        )
        .await
    }

    pub async fn search_release_groups(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<ReleaseGroup>> {
        let pool = self.pool.clone();
        let query_owned = query.to_string();
        cached_search(
            SearchCacheRepository::get_release_group_search(
                &self.pool,
                query,
                limit,
                offset,
                self.cache_ttl,
            ),
            self.mb_client
                .search_release_groups(query, limit, offset),
            move |rgs: &[ReleaseGroup]| {
                let rgs = rgs.to_vec();
                spawn_cache_task("release group search", move || async move {
                    SearchCacheRepository::cache_release_group_search(
                        &pool,
                        &query_owned,
                        limit,
                        offset,
                        &rgs,
                    )
                    .await
                });
            },
            "release_group",
            query,
        )
        .await
    }
}
