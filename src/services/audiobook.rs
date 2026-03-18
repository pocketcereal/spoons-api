use crate::audiobook::{Audiobook, Chapter};
use crate::cache::cached::{CacheTtlSeconds, cached_get_optional, cached_search};
use crate::db::repositories::{
    AudiobookRepository, ChapterRepository, SearchCacheRepository,
};
use crate::db::{DbPool, spawn_cache_task};
use crate::error::Result;
use crate::librivox::LibriVoxClient;

#[derive(Clone)]
pub struct AudiobookService {
    pool: DbPool,
    client: LibriVoxClient,
    cache_ttl: CacheTtlSeconds,
}

impl AudiobookService {
    pub fn new(pool: DbPool, client: LibriVoxClient, cache_ttl: CacheTtlSeconds) -> Self {
        Self {
            pool,
            client,
            cache_ttl,
        }
    }

    pub fn client(&self) -> &LibriVoxClient {
        &self.client
    }

    pub async fn get_audiobook(&self, id: i64) -> Result<Option<Audiobook>> {
        let pool = self.pool.clone();
        let cache_key = id.to_string();
        cached_get_optional(
            AudiobookRepository::get_cached(&self.pool, id, self.cache_ttl),
            async {
                self.client
                    .get_audiobook(id)
                    .await?
                    .ok_or_else(|| crate::error::AppError::NotFound(format!("Audiobook {} not found", id)))
            },
            move |audiobook: &Audiobook| {
                let audiobook = audiobook.clone();
                spawn_cache_task("audiobook", move || async move {
                    AudiobookRepository::upsert(&pool, &audiobook).await
                });
            },
            "audiobook",
            &cache_key,
        )
        .await
    }

    pub async fn search_audiobooks(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Audiobook>> {
        let pool = self.pool.clone();
        let query_owned = query.to_string();
        cached_search(
            SearchCacheRepository::get_audiobook_search(
                &self.pool,
                query,
                limit,
                offset,
                self.cache_ttl,
            ),
            self.client.search_audiobooks(query, limit, offset),
            move |audiobooks: &[Audiobook]| {
                let audiobooks = audiobooks.to_vec();
                spawn_cache_task("audiobook search", move || async move {
                    SearchCacheRepository::cache_audiobook_search(
                        &pool,
                        &query_owned,
                        limit,
                        offset,
                        &audiobooks,
                    )
                    .await
                });
            },
            "audiobook",
            query,
        )
        .await
    }

    pub async fn get_chapters(
        &self,
        audiobook_id: i64,
        limit: i32,
    ) -> Result<Vec<Chapter>> {
        let pool = self.pool.clone();
        let cache_key = audiobook_id.to_string();
        cached_search(
            ChapterRepository::get_cached_by_audiobook_id(
                &self.pool,
                audiobook_id,
                limit,
                self.cache_ttl,
            ),
            self.client.get_chapters(audiobook_id),
            move |chapters: &[Chapter]| {
                let chapters = chapters.to_vec();
                spawn_cache_task("chapters", move || async move {
                    ChapterRepository::upsert_many(&pool, &chapters).await
                });
            },
            "chapter",
            &cache_key,
        )
        .await
    }
}
