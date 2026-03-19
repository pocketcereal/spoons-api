use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::audiobook::{Audiobook, Chapter};
use crate::error::Result;
use crate::http::{ApiClient, ClientConfig, DEFAULT_API_TIMEOUT};
use crate::librivox::endpoints;

const DEFAULT_BASE_URL: &str = "https://librivox.org/api/feed";

#[derive(Debug, Clone)]
pub struct LibriVoxClient {
    client: ApiClient,
}

impl LibriVoxClient {
    pub fn new(base_url: &str) -> Result<Self> {
        let client = ClientConfig::new(base_url)
            .with_timeout(DEFAULT_API_TIMEOUT)
            .build()?;

        Ok(Self { client })
    }

    pub fn default_client() -> Result<Self> {
        Self::new(DEFAULT_BASE_URL)
    }

    pub(crate) async fn get_with_query<T: DeserializeOwned, Q: Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        self.client.get_with_query(path, query).await
    }

    pub async fn search_audiobooks(
        &self,
        title: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Audiobook>> {
        endpoints::search_audiobooks(self, title, limit, offset).await
    }

    pub async fn get_audiobook(&self, id: i64) -> Result<Option<Audiobook>> {
        endpoints::get_audiobook_by_id(self, id).await
    }

    pub async fn get_chapters(&self, audiobook_id: i64) -> Result<Vec<Chapter>> {
        endpoints::get_chapters(self, audiobook_id).await
    }

    pub async fn get_audiobooks_page(&self, limit: i32, offset: i32) -> Result<Vec<Audiobook>> {
        endpoints::get_audiobooks_page(self, limit, offset).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_construction() {
        let client = LibriVoxClient::new("https://librivox.org/api/feed");
        assert!(client.is_ok());
    }

    #[test]
    fn test_default_client() {
        let client = LibriVoxClient::default_client();
        assert!(client.is_ok());
    }
}
