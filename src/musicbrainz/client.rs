//! MusicBrainz API client.

use crate::error::Result;
use crate::http::{ApiClient, ClientConfig, DEFAULT_API_TIMEOUT};

use super::types::{Artist, Recording, Release, ReleaseGroup, SearchResult};

/// MusicBrainz API client.
#[derive(Clone)]
pub struct MusicBrainzClient {
    client: ApiClient,
}

impl MusicBrainzClient {
    /// Create a new MusicBrainz client.
    pub fn new(base_url: &str) -> Result<Self> {
        let client = ClientConfig::new(base_url)
            .with_timeout(DEFAULT_API_TIMEOUT)
            .build()?;

        Ok(Self { client })
    }

    /// Create a client with the default MusicBrainz API URL.
    pub fn default_client() -> Result<Self> {
        Self::new("https://musicbrainz.org/ws/2")
    }

    /// Search for artists.
    pub async fn search_artists(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Artist>> {
        let params = SearchParams {
            query: query.to_string(),
            limit,
            offset,
            fmt: "json".to_string(),
        };

        let result: SearchResult<Artist> = self.client.get_with_query("/artist", &params).await?;

        Ok(result.items())
    }

    /// Get an artist by ID.
    pub async fn get_artist(&self, id: &str) -> Result<Artist> {
        let path = format!("/artist/{}?fmt=json", id);
        self.client.get(&path).await
    }

    /// Search for releases.
    pub async fn search_releases(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Release>> {
        let params = SearchParams {
            query: query.to_string(),
            limit,
            offset,
            fmt: "json".to_string(),
        };

        let result: SearchResult<Release> = self.client.get_with_query("/release", &params).await?;

        Ok(result.items())
    }

    /// Get a release by ID.
    pub async fn get_release(&self, id: &str) -> Result<Release> {
        let path = format!("/release/{}?fmt=json", id);
        self.client.get(&path).await
    }

    /// Search for recordings.
    pub async fn search_recordings(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<Recording>> {
        let params = SearchParams {
            query: query.to_string(),
            limit,
            offset,
            fmt: "json".to_string(),
        };

        let result: SearchResult<Recording> =
            self.client.get_with_query("/recording", &params).await?;

        Ok(result.items())
    }

    /// Get a recording by ID.
    pub async fn get_recording(&self, id: &str) -> Result<Recording> {
        let path = format!("/recording/{}?inc=artist-credits&fmt=json", id);
        self.client.get(&path).await
    }

    /// Search for release groups.
    pub async fn search_release_groups(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<ReleaseGroup>> {
        let params = SearchParams {
            query: query.to_string(),
            limit,
            offset,
            fmt: "json".to_string(),
        };

        let result: SearchResult<ReleaseGroup> = self
            .client
            .get_with_query("/release-group", &params)
            .await?;

        Ok(result.items())
    }

    /// Get a release group by ID.
    pub async fn get_release_group(&self, id: &str) -> Result<ReleaseGroup> {
        let path = format!("/release-group/{}?fmt=json", id);
        self.client.get(&path).await
    }
}

/// Search query parameters.
#[derive(serde::Serialize)]
struct SearchParams {
    query: String,
    limit: i32,
    offset: i32,
    fmt: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = MusicBrainzClient::new("https://musicbrainz.org/ws/2");
        assert!(client.is_ok());
    }
}
