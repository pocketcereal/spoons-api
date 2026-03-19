//! Podcast service implementing cache-first patterns over PodcastIndex API.

use crate::cache::cached::{CacheTtlSeconds, cached_get_optional, cached_search};
use crate::db::repositories::{EpisodeRepository, PodcastRepository, SearchCacheRepository};
use crate::db::{DbPool, spawn_cache_task};
use crate::error::Result;
use crate::podcast::{Episode, Podcast};
use crate::podcast_index::PodcastIndexClient;

/// Podcast service with cache-first access to PodcastIndex.
#[derive(Clone)]
pub struct PodcastService {
    pool: DbPool,
    client: PodcastIndexClient,
    cache_ttl: CacheTtlSeconds,
}

impl PodcastService {
    pub fn new(pool: DbPool, client: PodcastIndexClient, cache_ttl: CacheTtlSeconds) -> Self {
        Self {
            pool,
            client,
            cache_ttl,
        }
    }

    /// Direct access to the underlying client for non-cached operations.
    pub fn client(&self) -> &PodcastIndexClient {
        &self.client
    }

    // === Podcasts ===

    pub async fn get_podcast(&self, feed_id: i64) -> Result<Option<Podcast>> {
        let pool = self.pool.clone();
        let cache_key = feed_id.to_string();
        cached_get_optional(
            PodcastRepository::get_cached(&self.pool, feed_id, self.cache_ttl),
            self.client.get_podcast(feed_id),
            move |podcast: &Podcast| {
                let podcast = podcast.clone();
                spawn_cache_task("podcast", move || async move {
                    PodcastRepository::upsert(&pool, &podcast).await
                });
            },
            "podcast",
            &cache_key,
        )
        .await
    }

    pub async fn search_podcasts(&self, query: &str, limit: i32) -> Result<Vec<Podcast>> {
        let pool = self.pool.clone();
        let query_owned = query.to_string();
        cached_search(
            SearchCacheRepository::get_podcast_search(&self.pool, query, limit, 0, self.cache_ttl),
            self.client.search_podcasts(query, limit),
            move |podcasts: &[Podcast]| {
                let podcasts = podcasts.to_vec();
                spawn_cache_task("podcast search", move || async move {
                    SearchCacheRepository::cache_podcast_search(
                        &pool,
                        &query_owned,
                        limit,
                        0,
                        &podcasts,
                    )
                    .await
                });
            },
            "podcast",
            query,
        )
        .await
    }

    pub async fn search_by_title(&self, title: &str, limit: i32) -> Result<Vec<Podcast>> {
        let pool = self.pool.clone();
        let title_owned = title.to_string();
        let cache_key = format!("title:{}", title);
        cached_search(
            SearchCacheRepository::get_podcast_search(
                &self.pool,
                &cache_key,
                limit,
                0,
                self.cache_ttl,
            ),
            self.client.search_by_title(title, limit),
            move |podcasts: &[Podcast]| {
                let podcasts = podcasts.to_vec();
                spawn_cache_task("podcast title search", move || async move {
                    SearchCacheRepository::cache_podcast_search(
                        &pool,
                        &format!("title:{}", title_owned),
                        limit,
                        0,
                        &podcasts,
                    )
                    .await
                });
            },
            "podcast",
            title,
        )
        .await
    }

    // === Episodes ===

    pub async fn get_episode(&self, episode_id: i64) -> Result<Option<Episode>> {
        let pool = self.pool.clone();
        let cache_key = episode_id.to_string();
        cached_get_optional(
            EpisodeRepository::get_cached(&self.pool, episode_id, self.cache_ttl),
            self.client.get_episode(episode_id),
            move |episode: &Episode| {
                let episode = episode.clone();
                spawn_cache_task("episode", move || async move {
                    EpisodeRepository::upsert(&pool, &episode).await
                });
            },
            "episode",
            &cache_key,
        )
        .await
    }

    pub async fn get_episodes(&self, feed_id: i64, limit: i32) -> Result<Vec<Episode>> {
        let pool = self.pool.clone();
        let cache_key = feed_id.to_string();
        cached_search(
            EpisodeRepository::get_cached_by_podcast_id(&self.pool, feed_id, limit, self.cache_ttl),
            self.client.get_episodes(feed_id, limit),
            move |episodes: &[Episode]| {
                let episodes = episodes.to_vec();
                spawn_cache_task("episodes", move || async move {
                    EpisodeRepository::upsert_many(&pool, &episodes).await
                });
            },
            "episode",
            &cache_key,
        )
        .await
    }
}
