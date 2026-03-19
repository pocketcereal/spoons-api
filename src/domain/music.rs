use crate::domain::DataSource;
use crate::error::Result;
use crate::graphql::types::{Artist, Track};

#[async_trait::async_trait]
pub trait MusicProvider: Send + Sync {
    fn source_id(&self) -> DataSource;
    async fn search_artists(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Artist>>;
    async fn search_tracks(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Track>>;
    async fn get_artist(&self, id: &str) -> Result<Option<Artist>>;
    async fn get_track(&self, id: &str) -> Result<Option<Track>>;
    async fn random_artists(&self, limit: i32) -> Result<Vec<Artist>>;
    async fn random_tracks(&self, limit: i32) -> Result<Vec<Track>>;
    async fn trending_tracks(&self, limit: i32) -> Result<Vec<Track>>;
}
