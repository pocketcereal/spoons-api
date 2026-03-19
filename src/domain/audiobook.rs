use crate::audiobook::AudiobookSource;
use crate::error::Result;
use crate::graphql::audiobook::{Audiobook, Chapter};

#[async_trait::async_trait]
pub trait AudiobookProvider: Send + Sync {
    fn source_id(&self) -> AudiobookSource;
    async fn search_audiobooks(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Audiobook>>;
    async fn get_audiobook(&self, id: i64) -> Result<Option<Audiobook>>;
    async fn get_chapters(&self, audiobook_id: i64, limit: i32) -> Result<Vec<Chapter>>;
    async fn random_audiobooks(&self, limit: i32) -> Result<Vec<Audiobook>>;
}
