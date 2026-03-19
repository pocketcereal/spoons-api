use serde::Serialize;
use std::time::Duration;

use crate::error::Result;
use crate::http::ApiClient;

use super::types::*;

const JAMENDO_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct JamendoClient {
    client: ApiClient,
    client_id: String,
}

#[derive(Debug, Serialize)]
struct SearchParams<'a> {
    client_id: &'a str,
    format: &'static str,
    search: &'a str,
    limit: i32,
    offset: i32,
}

#[derive(Debug, Serialize)]
struct ArtistSearchParams<'a> {
    client_id: &'a str,
    format: &'static str,
    namesearch: &'a str,
    limit: i32,
    offset: i32,
}

#[derive(Debug, Serialize)]
struct IdParams<'a> {
    client_id: &'a str,
    format: &'static str,
    id: &'a str,
}

#[derive(Debug, Serialize)]
struct PopularParams<'a> {
    client_id: &'a str,
    format: &'static str,
    order: &'static str,
    limit: i32,
}

impl JamendoClient {
    pub fn new(client_id: String, base_url: &str) -> Result<Self> {
        let client = ApiClient::new(base_url, JAMENDO_TIMEOUT)?;
        Ok(Self { client, client_id })
    }

    pub async fn search_tracks(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<JamendoTrack>> {
        let params = SearchParams {
            client_id: &self.client_id,
            format: "json",
            search: query,
            limit,
            offset,
        };
        let response: JamendoResponse<JamendoTrack> =
            self.client.get_with_query("/tracks", &params).await?;
        Ok(response.results)
    }

    pub async fn search_artists(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<JamendoArtist>> {
        let params = ArtistSearchParams {
            client_id: &self.client_id,
            format: "json",
            namesearch: query,
            limit,
            offset,
        };
        let response: JamendoResponse<JamendoArtist> =
            self.client.get_with_query("/artists", &params).await?;
        Ok(response.results)
    }

    pub async fn get_track(&self, id: &str) -> Result<Option<JamendoTrack>> {
        let params = IdParams {
            client_id: &self.client_id,
            format: "json",
            id,
        };
        let response: JamendoResponse<JamendoTrack> =
            self.client.get_with_query("/tracks", &params).await?;
        Ok(response.results.into_iter().next())
    }

    pub async fn get_artist(&self, id: &str) -> Result<Option<JamendoArtist>> {
        let params = IdParams {
            client_id: &self.client_id,
            format: "json",
            id,
        };
        let response: JamendoResponse<JamendoArtist> =
            self.client.get_with_query("/artists", &params).await?;
        Ok(response.results.into_iter().next())
    }

    pub async fn popular_tracks(&self, limit: i32) -> Result<Vec<JamendoTrack>> {
        let params = PopularParams {
            client_id: &self.client_id,
            format: "json",
            order: "popularity_total",
            limit,
        };
        let response: JamendoResponse<JamendoTrack> =
            self.client.get_with_query("/tracks", &params).await?;
        Ok(response.results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_construction() {
        let client =
            JamendoClient::new("test_client_id".to_string(), "https://api.jamendo.com/v3.0");
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.client_id, "test_client_id");
    }

    #[test]
    fn test_client_construction_with_trailing_slash() {
        let client = JamendoClient::new("my_id".to_string(), "https://api.jamendo.com/v3.0/");
        assert!(client.is_ok());
    }
}
