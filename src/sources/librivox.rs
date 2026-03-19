use crate::audiobook::AudiobookSource;
use crate::domain::AudiobookProvider;
use crate::error::Result;
use crate::graphql::audiobook::{Audiobook, Chapter};
use crate::graphql::helpers::random_sample;
use crate::services::AudiobookService;

const LIBRIVOX_MAX_OFFSET: i64 = 20_000;
const RANDOM_RETRY_ATTEMPTS: u32 = 3;

pub struct LibriVoxProvider {
    service: AudiobookService,
}

impl LibriVoxProvider {
    pub fn new(service: AudiobookService) -> Self {
        Self { service }
    }
}

#[async_trait::async_trait]
impl AudiobookProvider for LibriVoxProvider {
    fn source_id(&self) -> AudiobookSource {
        AudiobookSource::LibriVox
    }

    async fn search_audiobooks(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Audiobook>> {
        let results = self.service.search_audiobooks(query, limit, offset).await?;
        Ok(results.into_iter().map(Audiobook::from).collect())
    }

    async fn get_audiobook(&self, id: i64) -> Result<Option<Audiobook>> {
        let result = self.service.get_audiobook(id).await?;
        Ok(result.map(Audiobook::from))
    }

    async fn get_chapters(&self, audiobook_id: i64, limit: i32) -> Result<Vec<Chapter>> {
        let results = self.service.get_chapters(audiobook_id, limit).await?;
        Ok(results.into_iter().map(Chapter::from).collect())
    }

    async fn random_audiobooks(&self, limit: i32) -> Result<Vec<Audiobook>> {
        let fetch_limit = limit * 2;
        let mut offset =
            rand::Rng::gen_range(&mut rand::thread_rng(), 0..LIBRIVOX_MAX_OFFSET as i32);
        for _ in 0..RANDOM_RETRY_ATTEMPTS {
            let results = self
                .service
                .get_audiobooks_page(fetch_limit, offset)
                .await?;
            if !results.is_empty() {
                return Ok(random_sample(results, limit as usize)
                    .into_iter()
                    .map(Audiobook::from)
                    .collect());
            }
            offset /= 2;
        }
        Ok(Vec::new())
    }
}
