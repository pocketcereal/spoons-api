//! PodcastIndex API client implementation.
//!
//! This client uses the shared HTTP infrastructure from `crate::http` for
//! connection pooling, retry logic, and consistent error handling.
//! Authentication headers are generated per-request at this layer.

use reqwest::header::{HeaderMap, HeaderValue};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::config::PodcastIndexConfig;
use crate::error::{AppError, Result};
use crate::http::{ApiClient, ClientConfig, DEFAULT_API_TIMEOUT};
use crate::podcast::{Category, Episode, Podcast};
use crate::podcast_index::auth::PodcastIndexAuth;
use crate::podcast_index::endpoints;

/// Client for interacting with the PodcastIndex API.
///
/// Uses the shared `ApiClient` infrastructure for connection pooling,
/// automatic retry with exponential backoff, and consistent error handling.
/// Authentication headers (HMAC-SHA1) are generated fresh per request.
#[derive(Debug, Clone)]
pub struct PodcastIndexClient {
    client: ApiClient,
    auth: PodcastIndexAuth,
}

impl PodcastIndexClient {
    /// Creates a new PodcastIndex client from configuration.
    pub fn from_config(config: &PodcastIndexConfig) -> Result<Self> {
        let api_key = config.api_key.as_ref().ok_or_else(|| {
            crate::error::AppError::Config("PodcastIndex API key not configured".to_string())
        })?;
        let api_secret = config.api_secret.as_ref().ok_or_else(|| {
            crate::error::AppError::Config("PodcastIndex API secret not configured".to_string())
        })?;

        Self::with_base_url(api_key, api_secret, &config.base_url)
    }

    /// Creates a new PodcastIndex client with default base URL.
    pub fn new(api_key: &str, api_secret: &str) -> Result<Self> {
        Self::with_base_url(
            api_key,
            api_secret,
            &crate::config::PodcastIndexConfig::default().base_url,
        )
    }

    /// Creates a new PodcastIndex client with a custom base URL.
    pub fn with_base_url(api_key: &str, api_secret: &str, base_url: &str) -> Result<Self> {
        let auth = PodcastIndexAuth::new(api_key.to_string(), api_secret.to_string());

        let client = ClientConfig::new(base_url)
            .with_timeout(DEFAULT_API_TIMEOUT)
            .build()?;

        Ok(Self { client, auth })
    }

    /// Builds a `HeaderMap` with fresh authentication headers.
    fn auth_header_map(&self) -> Result<HeaderMap> {
        let headers = self.auth.generate_headers();
        let mut map = HeaderMap::with_capacity(3);
        let to_val = |v: &str| {
            HeaderValue::from_str(v)
                .map_err(|e| AppError::Config(format!("Invalid auth header value: {}", e)))
        };
        map.insert("X-Auth-Key", to_val(&headers.x_auth_key)?);
        map.insert("X-Auth-Date", to_val(&headers.x_auth_date)?);
        map.insert("Authorization", to_val(&headers.authorization)?);
        Ok(map)
    }

    /// Makes an authenticated GET request to the PodcastIndex API.
    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.client
            .get_with_headers(path, &(), self.auth_header_map()?)
            .await
    }

    /// Makes an authenticated GET request with query parameters to the PodcastIndex API.
    pub(crate) async fn get_with_query<T: DeserializeOwned, Q: Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        self.client
            .get_with_headers(path, query, self.auth_header_map()?)
            .await
    }

    // Public API methods

    /// Searches for podcasts using general term search.
    pub async fn search_podcasts(&self, query: &str, limit: i32) -> Result<Vec<Podcast>> {
        endpoints::search_podcasts(self, query, limit).await
    }

    /// Searches for podcasts by title.
    pub async fn search_by_title(&self, title: &str, limit: i32) -> Result<Vec<Podcast>> {
        endpoints::search_by_title(self, title, limit).await
    }

    /// Gets trending podcasts.
    pub async fn trending(&self, limit: i32, categories: Option<&[i32]>) -> Result<Vec<Podcast>> {
        endpoints::get_trending(self, limit, categories).await
    }

    /// Gets all available podcast categories.
    pub async fn categories(&self) -> Result<Vec<Category>> {
        endpoints::get_categories(self).await
    }

    /// Gets a podcast by its feed ID.
    pub async fn get_podcast(&self, feed_id: i64) -> Result<Podcast> {
        endpoints::get_podcast_by_feed_id(self, feed_id).await
    }

    /// Gets episodes for a podcast.
    pub async fn get_episodes(&self, feed_id: i64, limit: i32) -> Result<Vec<Episode>> {
        endpoints::get_episodes(self, feed_id, limit).await
    }

    /// Gets a single episode by its ID.
    pub async fn get_episode(&self, episode_id: i64) -> Result<Episode> {
        endpoints::get_episode_by_id(self, episode_id).await
    }

    /// Gets random episodes with optional filters.
    pub async fn random_episodes(
        &self,
        limit: i32,
        lang: Option<&str>,
        categories: Option<&[i32]>,
    ) -> Result<Vec<Episode>> {
        endpoints::get_random_episodes(self, limit, lang, categories).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_construction() {
        let client = PodcastIndexClient::new("test_key", "test_secret");
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_custom_base_url() {
        let custom_url = "https://test.example.com/api";
        let client = PodcastIndexClient::with_base_url("test_key", "test_secret", custom_url);
        assert!(client.is_ok());
    }
}
