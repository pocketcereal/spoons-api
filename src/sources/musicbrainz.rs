use crate::domain::{DataSource, MusicProvider};
use crate::error::{AppError, Result};
use crate::graphql::helpers::random_sample;
use crate::graphql::types::{Artist, Track};
use crate::services::MusicService;

const MUSICBRAINZ_MAX_OFFSET: i64 = 10_000;

pub struct MusicBrainzProvider {
    service: MusicService,
}

impl MusicBrainzProvider {
    pub fn new(service: MusicService) -> Self {
        Self { service }
    }
}

#[async_trait::async_trait]
impl MusicProvider for MusicBrainzProvider {
    fn source_id(&self) -> DataSource {
        DataSource::MusicBrainz
    }

    async fn search_artists(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Artist>> {
        let results = self.service.search_artists(query, limit, offset).await?;
        Ok(results.into_iter().map(|a| Artist::MusicBrainz(a.into())).collect())
    }

    async fn search_tracks(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Track>> {
        let results = self.service.search_recordings(query, limit, offset).await?;
        Ok(results.into_iter().map(|r| Track::MusicBrainz(r.into())).collect())
    }

    async fn get_artist(&self, id: &str) -> Result<Option<Artist>> {
        match self.service.get_artist(id).await {
            Ok(a) => Ok(Some(Artist::MusicBrainz(a.into()))),
            Err(AppError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_track(&self, id: &str) -> Result<Option<Track>> {
        match self.service.get_recording(id).await {
            Ok(r) => Ok(Some(Track::MusicBrainz(r.into()))),
            Err(AppError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn random_artists(&self, limit: i32) -> Result<Vec<Artist>> {
        let (count, _) = self
            .service
            .mb_client()
            .search_artists_with_count("*", 1, 0)
            .await?;
        if count == 0 {
            return Ok(vec![]);
        }
        let max_offset = count.min(MUSICBRAINZ_MAX_OFFSET) - 1;
        let offset = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=max_offset as i32);
        let (_, artists) = self
            .service
            .mb_client()
            .search_artists_with_count("*", limit, offset)
            .await?;
        Ok(random_sample(artists, limit as usize)
            .into_iter()
            .map(|a| Artist::MusicBrainz(a.into()))
            .collect())
    }

    async fn random_tracks(&self, limit: i32) -> Result<Vec<Track>> {
        let (count, _) = self
            .service
            .mb_client()
            .search_recordings_with_count("*", 1, 0)
            .await?;
        if count == 0 {
            return Ok(vec![]);
        }
        let max_offset = count.min(MUSICBRAINZ_MAX_OFFSET) - 1;
        let offset = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=max_offset as i32);
        let (_, recordings) = self
            .service
            .mb_client()
            .search_recordings_with_count("*", limit, offset)
            .await?;
        Ok(random_sample(recordings, limit as usize)
            .into_iter()
            .map(|r| Track::MusicBrainz(r.into()))
            .collect())
    }
}
