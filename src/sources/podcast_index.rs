use crate::domain::PodcastProvider;
use crate::error::Result;
use crate::graphql::podcast::{Category, Episode, Podcast};
use crate::podcast::PodcastSource;
use crate::services::PodcastService;

pub struct PodcastIndexProvider {
    service: PodcastService,
}

impl PodcastIndexProvider {
    pub fn new(service: PodcastService) -> Self {
        Self { service }
    }
}

impl PodcastProvider for PodcastIndexProvider {
    fn source_id(&self) -> PodcastSource {
        PodcastSource::PodcastIndex
    }

    async fn search_podcasts(&self, query: &str, limit: i32) -> Result<Vec<Podcast>> {
        let results = self.service.search_podcasts(query, limit).await?;
        Ok(results.into_iter().map(Podcast::from).collect())
    }

    async fn search_by_title(&self, title: &str, limit: i32) -> Result<Vec<Podcast>> {
        let results = self.service.search_by_title(title, limit).await?;
        Ok(results.into_iter().map(Podcast::from).collect())
    }

    async fn get_podcast(&self, id: i64) -> Result<Option<Podcast>> {
        let result = self.service.get_podcast(id).await?;
        Ok(result.map(Podcast::from))
    }

    async fn get_episodes(&self, podcast_id: i64, limit: i32) -> Result<Vec<Episode>> {
        let results = self.service.get_episodes(podcast_id, limit).await?;
        Ok(results.into_iter().map(Episode::from).collect())
    }

    async fn get_episode(&self, id: i64) -> Result<Option<Episode>> {
        let result = self.service.get_episode(id).await?;
        Ok(result.map(Episode::from))
    }

    async fn trending(&self, limit: i32, categories: Option<&[i32]>) -> Result<Vec<Podcast>> {
        let results = self.service.client().trending(limit, categories).await?;
        Ok(results.into_iter().map(Podcast::from).collect())
    }

    async fn categories(&self) -> Result<Vec<Category>> {
        let results = self.service.client().categories().await?;
        Ok(results.into_iter().map(Category::from).collect())
    }

    async fn random_episodes(
        &self,
        limit: i32,
        language: Option<&str>,
        categories: Option<&[i32]>,
    ) -> Result<Vec<Episode>> {
        let results = self
            .service
            .client()
            .random_episodes(limit, language, categories)
            .await?;
        Ok(results.into_iter().map(Episode::from).collect())
    }
}
