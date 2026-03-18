use std::collections::HashSet;

use crate::audius::AudiusClient;
use crate::domain::{DataSource, MusicProvider};
use crate::error::Result;
use crate::graphql::helpers::random_sample;
use crate::graphql::types::{Artist, Track};

pub struct AudiusProvider {
    client: AudiusClient,
}

impl AudiusProvider {
    pub fn new(client: AudiusClient) -> Self {
        Self { client }
    }
}

impl MusicProvider for AudiusProvider {
    fn source_id(&self) -> DataSource {
        DataSource::Audius
    }

    async fn search_artists(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Artist>> {
        let users = self.client.search_users(query, limit, offset).await?;
        Ok(users.into_iter().map(|u| Artist::Audius(u.into())).collect())
    }

    async fn search_tracks(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Track>> {
        let tracks = self.client.search_tracks(query, limit, offset).await?;
        Ok(tracks.into_iter().map(|t| Track::Audius(t.into())).collect())
    }

    async fn get_artist(&self, id: &str) -> Result<Option<Artist>> {
        let user = self.client.get_user(id).await?;
        Ok(Some(Artist::Audius(user.into())))
    }

    async fn get_track(&self, id: &str) -> Result<Option<Track>> {
        let track = self.client.get_track(id).await?;
        Ok(Some(Track::Audius(track.into())))
    }

    async fn random_artists(&self, limit: i32) -> Result<Vec<Artist>> {
        let tracks = self.client.trending_tracks(100).await?;
        let mut seen = HashSet::new();
        let unique_users: Vec<_> = tracks
            .into_iter()
            .filter_map(|t| t.user)
            .filter(|u| seen.insert(u.id.clone()))
            .collect();
        let sampled = random_sample(unique_users, limit as usize);
        Ok(sampled.into_iter().map(|u| Artist::Audius(u.into())).collect())
    }

    async fn random_tracks(&self, limit: i32) -> Result<Vec<Track>> {
        let pool_size = limit * 3;
        let tracks = self.client.trending_tracks(pool_size).await?;
        let sampled = random_sample(tracks, limit as usize);
        Ok(sampled.into_iter().map(|t| Track::Audius(t.into())).collect())
    }
}
