use std::collections::HashSet;

use crate::domain::{DataSource, MusicProvider};
use crate::error::Result;
use crate::graphql::helpers::random_sample;
use crate::graphql::types::{Artist, Track};
use crate::jamendo::JamendoClient;

pub struct JamendoProvider {
    client: JamendoClient,
}

impl JamendoProvider {
    pub fn new(client: JamendoClient) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl MusicProvider for JamendoProvider {
    fn source_id(&self) -> DataSource {
        DataSource::Jamendo
    }

    async fn search_artists(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Artist>> {
        let results = self.client.search_artists(query, limit, offset).await?;
        Ok(results
            .into_iter()
            .map(|a| Artist::Jamendo(a.into()))
            .collect())
    }

    async fn search_tracks(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Track>> {
        let results = self.client.search_tracks(query, limit, offset).await?;
        Ok(results
            .into_iter()
            .map(|t| Track::Jamendo(t.into()))
            .collect())
    }

    async fn get_artist(&self, id: &str) -> Result<Option<Artist>> {
        let result = self.client.get_artist(id).await?;
        Ok(result.map(|a| Artist::Jamendo(a.into())))
    }

    async fn get_track(&self, id: &str) -> Result<Option<Track>> {
        let result = self.client.get_track(id).await?;
        Ok(result.map(|t| Track::Jamendo(t.into())))
    }

    async fn random_artists(&self, limit: i32) -> Result<Vec<Artist>> {
        let tracks = self.client.popular_tracks(limit * 3).await?;
        let mut seen = HashSet::new();
        let artists: Vec<Artist> = tracks
            .into_iter()
            .filter(|t| seen.insert(t.artist_id.clone()))
            .map(|t| {
                Artist::Jamendo(crate::graphql::types::JamendoArtist {
                    id: DataSource::Jamendo.format_id(&t.artist_id),
                    name: t.artist_name,
                    source_id: t.artist_id,
                    image_url: t.image,
                    website: None,
                })
            })
            .collect();
        Ok(random_sample(artists, limit as usize))
    }

    async fn random_tracks(&self, limit: i32) -> Result<Vec<Track>> {
        let tracks = self.client.popular_tracks(limit * 2).await?;
        let tracks: Vec<Track> = tracks
            .into_iter()
            .map(|t| Track::Jamendo(t.into()))
            .collect();
        Ok(random_sample(tracks, limit as usize))
    }

    async fn trending_tracks(&self, limit: i32) -> Result<Vec<Track>> {
        let tracks = self.client.popular_tracks(limit).await?;
        Ok(tracks
            .into_iter()
            .map(|t| Track::Jamendo(t.into()))
            .collect())
    }
}
