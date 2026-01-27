//! PodcastIndex API client implementation.

use crate::error::{AppError, Result};
use crate::podcast_index::auth::PodcastIndexAuth;
use reqwest::Client;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Default base URL for the PodcastIndex API.
const DEFAULT_BASE_URL: &str = "https://api.podcastindex.org/api/1.0";

/// Client for interacting with the PodcastIndex API.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PodcastIndexClient {
    client: Client,
    base_url: String,
    auth: PodcastIndexAuth,
}

impl PodcastIndexClient {
    /// Creates a new PodcastIndex client with the default base URL.
    pub fn new(api_key: &str, api_secret: &str) -> Result<Self> {
        Self::with_base_url(api_key, api_secret, DEFAULT_BASE_URL)
    }

    /// Creates a new PodcastIndex client with a custom base URL.
    pub fn with_base_url(api_key: &str, api_secret: &str, base_url: &str) -> Result<Self> {
        let client = Client::builder()
            .build()
            .map_err(|e| AppError::ExternalApi(e.to_string()))?;

        let auth = PodcastIndexAuth::new(api_key.to_string(), api_secret.to_string());

        Ok(Self {
            client,
            base_url: base_url.to_string(),
            auth,
        })
    }

    /// Makes an authenticated GET request to the PodcastIndex API.
    #[allow(dead_code)]
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.get_with_query(path, &()).await
    }

    /// Makes an authenticated GET request with query parameters to the PodcastIndex API.
    #[allow(dead_code)]
    async fn get_with_query<T: DeserializeOwned, Q: Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let headers = self.auth.generate_headers();

        let response = self
            .client
            .get(&url)
            .header("X-Auth-Key", headers.x_auth_key)
            .header("X-Auth-Date", headers.x_auth_date)
            .header("Authorization", headers.authorization)
            .header("User-Agent", headers.user_agent)
            .query(query)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::ExternalApi(format!(
                "PodcastIndex API error ({}): {}",
                status, error_text
            )));
        }

        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_construction() {
        let client = PodcastIndexClient::new("test_key", "test_secret");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn test_client_with_custom_base_url() {
        let custom_url = "https://test.example.com/api";
        let client = PodcastIndexClient::with_base_url("test_key", "test_secret", custom_url);
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.base_url, custom_url);
    }
}
