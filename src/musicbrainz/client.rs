use crate::error::Result;
use crate::http::{ApiClient, ClientConfig, DEFAULT_API_TIMEOUT};

use super::types::{Artist, Recording, Release, ReleaseGroup, SearchResult};

#[derive(Clone)]
pub struct MusicBrainzClient {
    client: ApiClient,
}

impl MusicBrainzClient {
    pub fn new(base_url: &str) -> Result<Self> {
        let client = ClientConfig::new(base_url)
            .with_timeout(DEFAULT_API_TIMEOUT)
            .build()?;

        Ok(Self { client })
    }

    pub fn default_client() -> Result<Self> {
        Self::new("https://musicbrainz.org/ws/2")
    }

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

    pub async fn get_artist(&self, id: &str) -> Result<Artist> {
        let path = format!("/artist/{}", id);
        self.client.get_with_query(&path, &FmtParam::default()).await
    }

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

    pub async fn get_release(&self, id: &str) -> Result<Release> {
        let path = format!("/release/{}", id);
        self.client.get_with_query(&path, &FmtParam::default()).await
    }

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

    pub async fn get_recording(&self, id: &str) -> Result<Recording> {
        let path = format!("/recording/{}", id);
        self.client.get_with_query(&path, &RecordingParams::default()).await
    }

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

    pub async fn get_release_group(&self, id: &str) -> Result<ReleaseGroup> {
        let path = format!("/release-group/{}", id);
        self.client.get_with_query(&path, &FmtParam::default()).await
    }
}

#[derive(serde::Serialize)]
struct SearchParams {
    query: String,
    limit: i32,
    offset: i32,
    fmt: String,
}

#[derive(serde::Serialize)]
struct FmtParam {
    fmt: String,
}

impl Default for FmtParam {
    fn default() -> Self {
        Self {
            fmt: "json".to_string(),
        }
    }
}

#[derive(serde::Serialize)]
struct RecordingParams {
    fmt: String,
    inc: String,
}

impl Default for RecordingParams {
    fn default() -> Self {
        Self {
            fmt: "json".to_string(),
            inc: "artist-credits".to_string(),
        }
    }
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
