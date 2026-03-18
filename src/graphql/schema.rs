use async_graphql::{
    Context, EmptyMutation, EmptySubscription, ErrorExtensions, MergedObject, Object, Schema,
};
use std::sync::Arc;
use std::time::Duration;

use crate::audius::AudiusClient;
use crate::domain::DataSource;
use crate::error::AppError;
use crate::services::{AudiobookService, MusicService, PodcastService};

use super::audiobook::AudiobookQuery;
use super::unified::UnifiedQuery;
use super::helpers::random_sample;
use super::podcast::PodcastQuery;
use super::types::{Artist, Track};

type GqlResult<T> = std::result::Result<T, async_graphql::Error>;

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(Clone)]
pub struct AppContext {
    pub music: MusicService,
    pub podcast: Option<PodcastService>,
    pub audiobook: Option<AudiobookService>,
}

pub fn build_schema(app_context: AppContext) -> AppSchema {
    Schema::build(QueryRoot::default(), EmptyMutation, EmptySubscription)
        .data(Arc::new(app_context))
        .limit_depth(10)
        .finish()
}

pub(crate) fn get_app_context<'a>(
    ctx: &'a Context<'_>,
) -> Result<&'a Arc<AppContext>, async_graphql::Error> {
    ctx.data::<Arc<AppContext>>().map_err(|_| {
        AppError::Internal(anyhow::anyhow!("Application context not configured")).extend()
    })
}

const MAX_QUERY_LENGTH: usize = 500;

pub(crate) fn validate_query(query: &str) -> GqlResult<String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("Search query cannot be empty".to_string()).extend());
    }
    if trimmed.len() > MAX_QUERY_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "Search query too long (max {} characters)",
            MAX_QUERY_LENGTH
        ))
        .extend());
    }
    Ok(trimmed.to_string())
}

const MAX_ID_LENGTH: usize = 64;

pub(crate) fn validate_id(id: &str) -> GqlResult<String> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("ID cannot be empty".to_string()).extend());
    }
    if trimmed.len() > MAX_ID_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "ID too long (max {} characters)",
            MAX_ID_LENGTH
        ))
        .extend());
    }
    Ok(trimmed.to_string())
}

const MAX_SEARCH_LIMIT: i32 = 100;
const MAX_SEARCH_OFFSET: i32 = 10000;

pub(crate) fn clamp_limit(limit: i32) -> i32 {
    limit.clamp(1, MAX_SEARCH_LIMIT)
}

fn clamp_offset(offset: i32) -> i32 {
    offset.clamp(0, MAX_SEARCH_OFFSET)
}

fn require_audius_client(app_ctx: &AppContext) -> GqlResult<&AudiusClient> {
    app_ctx
        .music
        .audius_client()
        .ok_or_else(|| {
            AppError::FeatureDisabled(
                "Audius is not configured. Set audius in config.yaml.".to_string(),
            )
            .extend()
        })
}

pub(crate) fn require_podcast_service(app_ctx: &AppContext) -> GqlResult<&PodcastService> {
    app_ctx
        .podcast
        .as_ref()
        .ok_or_else(|| {
            AppError::FeatureDisabled(
                "PodcastIndex is not configured. Set podcast_index in config.yaml.".to_string(),
            )
            .extend()
        })
}

pub(crate) fn require_audiobook_service(app_ctx: &AppContext) -> GqlResult<&AudiobookService> {
    app_ctx
        .audiobook
        .as_ref()
        .ok_or_else(|| {
            AppError::FeatureDisabled(
                "LibriVox is not configured. Set librivox in config.yaml.".to_string(),
            )
            .extend()
        })
}

pub(crate) const SOURCE_QUERY_TIMEOUT: Duration = Duration::from_secs(10);

/// Searches multiple sources in parallel, combining results and logging failures.
pub(crate) async fn search_sources<T, MbFut, AudiusFut>(
    source: Option<DataSource>,
    mb_search: MbFut,
    audius_search: AudiusFut,
    entity_name: &str,
) -> GqlResult<Vec<T>>
where
    MbFut: std::future::Future<Output = Result<Vec<T>, AppError>>,
    AudiusFut: std::future::Future<Output = Result<Vec<T>, AppError>>,
{
    match source {
        Some(DataSource::MusicBrainz) => mb_search.await.map_err(|e| e.extend()),
        Some(DataSource::Audius) => audius_search.await.map_err(|e| e.extend()),
        Some(DataSource::Jamendo) => Err(AppError::FeatureDisabled("Jamendo".into()).extend()),
        None => {
            let (mb_results, audius_results) = tokio::join!(
                tokio::time::timeout(SOURCE_QUERY_TIMEOUT, mb_search),
                tokio::time::timeout(SOURCE_QUERY_TIMEOUT, audius_search),
            );

            let mut combined = Vec::new();

            match mb_results {
                Ok(Ok(items)) => combined.extend(items),
                Ok(Err(e)) => {
                    tracing::warn!(error = %e, "MusicBrainz {} search failed", entity_name)
                }
                Err(_) => tracing::warn!("MusicBrainz {} search timed out", entity_name),
            }

            match audius_results {
                Ok(Ok(items)) => combined.extend(items),
                Ok(Err(e)) => tracing::warn!(error = %e, "Audius {} search failed", entity_name),
                Err(_) => tracing::warn!("Audius {} search timed out", entity_name),
            }

            Ok(combined)
        }
    }
}

#[derive(Default)]
pub struct MusicQuery;

#[Object]
impl MusicQuery {
    async fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    async fn search_artists(
        &self,
        ctx: &Context<'_>,
        query: String,
        source: Option<DataSource>,
        #[graphql(default = 25)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> GqlResult<Vec<Artist>> {
        let query = validate_query(&query)?;
        let app_ctx = get_app_context(ctx)?;
        let limit = clamp_limit(limit);
        let offset = clamp_offset(offset);

        search_sources(
            source,
            search_musicbrainz_artists(app_ctx, &query, limit, offset),
            search_audius_artists(app_ctx, &query, limit, offset),
            "artist",
        )
        .await
    }

    async fn artist(&self, ctx: &Context<'_>, id: String, source: DataSource) -> GqlResult<Artist> {
        let id = validate_id(&id)?;
        let app_ctx = get_app_context(ctx)?;

        match source {
            DataSource::MusicBrainz => {
                let artist = app_ctx
                    .music
                    .get_artist(&id)
                    .await
                    .map_err(|e| e.extend())?;
                Ok(Artist::MusicBrainz(artist.into()))
            }
            DataSource::Audius => {
                let client = require_audius_client(app_ctx)?;
                let user = client.get_user(&id).await.map_err(|e| e.extend())?;
                Ok(Artist::Audius(user.into()))
            }
            DataSource::Jamendo => Err(AppError::FeatureDisabled("Jamendo".into()).extend()),
        }
    }

    async fn search_tracks(
        &self,
        ctx: &Context<'_>,
        query: String,
        source: Option<DataSource>,
        #[graphql(default = 25)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> GqlResult<Vec<Track>> {
        let query = validate_query(&query)?;
        let app_ctx = get_app_context(ctx)?;
        let limit = clamp_limit(limit);
        let offset = clamp_offset(offset);

        search_sources(
            source,
            search_musicbrainz_tracks(app_ctx, &query, limit, offset),
            search_audius_tracks(app_ctx, &query, limit, offset),
            "track",
        )
        .await
    }

    async fn random_tracks(
        &self,
        ctx: &Context<'_>,
        source: Option<DataSource>,
        #[graphql(default = 10)] limit: i32,
    ) -> GqlResult<Vec<Track>> {
        let app_ctx = get_app_context(ctx)?;
        let limit = clamp_limit(limit);

        search_sources(
            source,
            random_musicbrainz_tracks(app_ctx, limit),
            random_audius_tracks(app_ctx, limit),
            "random track",
        )
        .await
    }

    async fn random_artists(
        &self,
        ctx: &Context<'_>,
        source: Option<DataSource>,
        #[graphql(default = 10)] limit: i32,
    ) -> GqlResult<Vec<Artist>> {
        let app_ctx = get_app_context(ctx)?;
        let limit = clamp_limit(limit);

        search_sources(
            source,
            random_musicbrainz_artists(app_ctx, limit),
            random_audius_artists(app_ctx, limit),
            "random artist",
        )
        .await
    }

    async fn track(&self, ctx: &Context<'_>, id: String, source: DataSource) -> GqlResult<Track> {
        let id = validate_id(&id)?;
        let app_ctx = get_app_context(ctx)?;

        match source {
            DataSource::MusicBrainz => {
                let recording = app_ctx
                    .music
                    .get_recording(&id)
                    .await
                    .map_err(|e| e.extend())?;
                Ok(Track::MusicBrainz(recording.into()))
            }
            DataSource::Audius => {
                let client = require_audius_client(app_ctx)?;
                let track = client.get_track(&id).await.map_err(|e| e.extend())?;
                Ok(Track::Audius(track.into()))
            }
            DataSource::Jamendo => Err(AppError::FeatureDisabled("Jamendo".into()).extend()),
        }
    }
}

#[derive(MergedObject, Default)]
pub struct QueryRoot(MusicQuery, PodcastQuery, AudiobookQuery, UnifiedQuery);

pub(crate) async fn search_musicbrainz_artists(
    app_ctx: &AppContext,
    query: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Artist>, AppError> {
    let artists = app_ctx.music.search_artists(query, limit, offset).await?;
    Ok(artists
        .into_iter()
        .map(|a| Artist::MusicBrainz(a.into()))
        .collect())
}

pub(crate) async fn search_audius_artists(
    app_ctx: &AppContext,
    query: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Artist>, AppError> {
    let client = match app_ctx.music.audius_client() {
        Some(c) => c,
        None => return Ok(Vec::new()),
    };

    let users = client.search_users(query, limit, offset).await?;
    Ok(users
        .into_iter()
        .map(|u| Artist::Audius(u.into()))
        .collect())
}

pub(crate) async fn search_musicbrainz_tracks(
    app_ctx: &AppContext,
    query: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Track>, AppError> {
    let recordings = app_ctx
        .music
        .search_recordings(query, limit, offset)
        .await?;
    Ok(recordings
        .into_iter()
        .map(|r| Track::MusicBrainz(r.into()))
        .collect())
}

pub(crate) async fn search_audius_tracks(
    app_ctx: &AppContext,
    query: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Track>, AppError> {
    let client = match app_ctx.music.audius_client() {
        Some(c) => c,
        None => return Ok(Vec::new()),
    };

    let tracks = client.search_tracks(query, limit, offset).await?;
    Ok(tracks
        .into_iter()
        .map(|t| Track::Audius(t.into()))
        .collect())
}

const MUSICBRAINZ_MAX_OFFSET: i64 = 10_000;

pub(crate) async fn random_musicbrainz_tracks(
    app_ctx: &AppContext,
    limit: i32,
) -> Result<Vec<Track>, AppError> {
    let (count, _) = app_ctx
        .music
        .mb_client()
        .search_recordings_with_count("*", 1, 0)
        .await?;

    if count == 0 {
        return Ok(Vec::new());
    }

    let max_offset = count.min(MUSICBRAINZ_MAX_OFFSET) - 1;
    let offset = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=max_offset as i32);

    let recordings = app_ctx
        .music
        .mb_client()
        .search_recordings_with_count("*", limit, offset as i32)
        .await?
        .1;

    Ok(recordings
        .into_iter()
        .map(|r| Track::MusicBrainz(r.into()))
        .collect())
}

pub(crate) async fn random_musicbrainz_artists(
    app_ctx: &AppContext,
    limit: i32,
) -> Result<Vec<Artist>, AppError> {
    let (count, _) = app_ctx
        .music
        .mb_client()
        .search_artists_with_count("*", 1, 0)
        .await?;

    if count == 0 {
        return Ok(Vec::new());
    }

    let max_offset = count.min(MUSICBRAINZ_MAX_OFFSET) - 1;
    let offset = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=max_offset as i32);

    let artists = app_ctx
        .music
        .mb_client()
        .search_artists_with_count("*", limit, offset as i32)
        .await?
        .1;

    Ok(artists
        .into_iter()
        .map(|a| Artist::MusicBrainz(a.into()))
        .collect())
}

pub(crate) async fn random_audius_tracks(
    app_ctx: &AppContext,
    limit: i32,
) -> Result<Vec<Track>, AppError> {
    let client = match app_ctx.music.audius_client() {
        Some(c) => c,
        None => return Ok(Vec::new()),
    };

    let pool_size = limit * 3;
    let tracks = client.trending_tracks(pool_size).await?;
    let sampled = random_sample(tracks, limit as usize);

    Ok(sampled
        .into_iter()
        .map(|t| Track::Audius(t.into()))
        .collect())
}

pub(crate) async fn random_audius_artists(
    app_ctx: &AppContext,
    limit: i32,
) -> Result<Vec<Artist>, AppError> {
    let client = match app_ctx.music.audius_client() {
        Some(c) => c,
        None => return Ok(Vec::new()),
    };

    let tracks = client.trending_tracks(100).await?;

    let mut seen = std::collections::HashSet::new();
    let unique_users: Vec<_> = tracks
        .into_iter()
        .filter_map(|t| t.user)
        .filter(|u| seen.insert(u.id.clone()))
        .collect();

    let sampled = random_sample(unique_users, limit as usize);

    Ok(sampled
        .into_iter()
        .map(|u| Artist::Audius(u.into()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{DbConfig, create_pool};
    use crate::musicbrainz::MusicBrainzClient;

    #[test]
    fn test_schema_builds() {
        let client = MusicBrainzClient::new("https://musicbrainz.org/ws/2")
            .expect("Failed to create MusicBrainz client");

        let db_url = match std::env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => return,
        };

        let db_config = DbConfig {
            url: db_url,
            max_connections: 1,
        };

        let pool = create_pool(&db_config).expect("Failed to create database pool");

        let app_context = AppContext {
            music: MusicService::new(pool, client, None, 3600),
            podcast: None,
            audiobook: None,
        };

        let _schema = build_schema(app_context);
    }
}
