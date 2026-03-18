use crate::error::Result;
use crate::graphql::podcast::{Category, Episode, Podcast};
use crate::podcast::PodcastSource;

pub trait PodcastProvider: Send + Sync {
    fn source_id(&self) -> PodcastSource;
    async fn search_podcasts(&self, query: &str, limit: i32) -> Result<Vec<Podcast>>;
    async fn search_by_title(&self, title: &str, limit: i32) -> Result<Vec<Podcast>>;
    async fn get_podcast(&self, id: i64) -> Result<Option<Podcast>>;
    async fn get_episodes(&self, podcast_id: i64, limit: i32) -> Result<Vec<Episode>>;
    async fn get_episode(&self, id: i64) -> Result<Option<Episode>>;
    async fn trending(&self, limit: i32, categories: Option<&[i32]>) -> Result<Vec<Podcast>>;
    async fn categories(&self) -> Result<Vec<Category>>;
    async fn random_episodes(&self, limit: i32, language: Option<&str>, categories: Option<&[i32]>) -> Result<Vec<Episode>>;
}
