//! GraphQL schema definition and query handlers.

use async_graphql::{
    Context, EmptyMutation, EmptySubscription, ErrorExtensions, MergedObject, Object, Schema,
};
use std::sync::Arc;
use std::time::Duration;

use crate::audius::AudiusClient;
use crate::db::{DbPool, MusicRepository};
use crate::domain::DataSource;
use crate::error::AppError;
use crate::musicbrainz::MusicBrainzClient;
use crate::podcast_index::PodcastIndexClient;

use super::podcast::PodcastQuery;
use super::types::{Artist, Track};

/// Result type alias for GraphQL resolvers that converts AppError to async_graphql::Error
type GqlResult<T> = std::result::Result<T, async_graphql::Error>;

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(Clone)]
pub struct AppContext {
    pub db_pool: DbPool,
    pub musicbrainz_client: MusicBrainzClient,
    pub audius_client: Option<AudiusClient>,
    pub podcast_index_client: Option<PodcastIndexClient>,
    pub cache_ttl_seconds: i64,
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

/// Maximum allowed length for search query strings.
const MAX_QUERY_LENGTH: usize = 500;

/// Validates and normalizes a search query string.
/// Trims whitespace, enforces max length, and rejects empty strings.
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

/// Maximum allowed limit for search queries.
const MAX_SEARCH_LIMIT: i32 = 100;

/// Maximum allowed offset for search queries.
const MAX_SEARCH_OFFSET: i32 = 10000;

/// Clamps the limit to the maximum allowed value.
pub(crate) fn clamp_limit(limit: i32) -> i32 {
    limit.clamp(1, MAX_SEARCH_LIMIT)
}

/// Clamps the offset to valid bounds (0 to MAX_SEARCH_OFFSET).
fn clamp_offset(offset: i32) -> i32 {
    offset.clamp(0, MAX_SEARCH_OFFSET)
}

/// Returns the Audius client or an error if not configured.
fn require_audius_client(app_ctx: &AppContext) -> GqlResult<&AudiusClient> {
    app_ctx
        .audius_client
        .as_ref()
        .ok_or_else(|| {
            AppError::FeatureDisabled(
                "Audius is not configured. Set audius in config.yaml.".to_string(),
            )
            .extend()
        })
}

/// Returns the PodcastIndex client or an error if not configured.
pub(crate) fn require_podcast_index_client(app_ctx: &AppContext) -> GqlResult<&PodcastIndexClient> {
    app_ctx
        .podcast_index_client
        .as_ref()
        .ok_or_else(|| {
            AppError::FeatureDisabled(
                "PodcastIndex is not configured. Set podcast_index in config.yaml.".to_string(),
            )
            .extend()
        })
}

/// Timeout for individual source queries when searching in parallel.
const SOURCE_QUERY_TIMEOUT: Duration = Duration::from_secs(10);

/// Searches multiple sources in parallel, combining results and logging failures.
async fn search_sources<T, MbFut, AudiusFut>(
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
        let app_ctx = get_app_context(ctx)?;

        match source {
            DataSource::MusicBrainz => {
                let artist = MusicRepository::get_artist(
                    &app_ctx.db_pool,
                    &app_ctx.musicbrainz_client,
                    &id,
                    app_ctx.cache_ttl_seconds,
                )
                .await
                .map_err(|e| e.extend())?;
                Ok(Artist::MusicBrainz(artist.into()))
            }
            DataSource::Audius => {
                let client = require_audius_client(app_ctx)?;
                let user = client.get_user(&id).await.map_err(|e| e.extend())?;
                Ok(Artist::Audius(user.into()))
            }
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

    async fn track(&self, ctx: &Context<'_>, id: String, source: DataSource) -> GqlResult<Track> {
        let app_ctx = get_app_context(ctx)?;

        match source {
            DataSource::MusicBrainz => {
                let recording = MusicRepository::get_recording(
                    &app_ctx.db_pool,
                    &app_ctx.musicbrainz_client,
                    &id,
                    app_ctx.cache_ttl_seconds,
                )
                .await
                .map_err(|e| e.extend())?;
                Ok(Track::MusicBrainz(recording.into()))
            }
            DataSource::Audius => {
                let client = require_audius_client(app_ctx)?;
                let track = client.get_track(&id).await.map_err(|e| e.extend())?;
                Ok(Track::Audius(track.into()))
            }
        }
    }
}

#[derive(MergedObject, Default)]
pub struct QueryRoot(MusicQuery, PodcastQuery);

async fn search_musicbrainz_artists(
    app_ctx: &AppContext,
    query: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Artist>, AppError> {
    let artists = MusicRepository::search_artists(
        &app_ctx.db_pool,
        &app_ctx.musicbrainz_client,
        query,
        limit,
        offset,
        app_ctx.cache_ttl_seconds,
    )
    .await?;

    Ok(artists
        .into_iter()
        .map(|a| Artist::MusicBrainz(a.into()))
        .collect())
}

async fn search_audius_artists(
    app_ctx: &AppContext,
    query: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Artist>, AppError> {
    let client = match &app_ctx.audius_client {
        Some(c) => c,
        None => return Ok(Vec::new()),
    };

    let users = client.search_users(query, limit, offset).await?;
    Ok(users
        .into_iter()
        .map(|u| Artist::Audius(u.into()))
        .collect())
}

async fn search_musicbrainz_tracks(
    app_ctx: &AppContext,
    query: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Track>, AppError> {
    let recordings = MusicRepository::search_recordings(
        &app_ctx.db_pool,
        &app_ctx.musicbrainz_client,
        query,
        limit,
        offset,
        app_ctx.cache_ttl_seconds,
    )
    .await?;

    Ok(recordings
        .into_iter()
        .map(|r| Track::MusicBrainz(r.into()))
        .collect())
}

async fn search_audius_tracks(
    app_ctx: &AppContext,
    query: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Track>, AppError> {
    let client = match &app_ctx.audius_client {
        Some(c) => c,
        None => return Ok(Vec::new()),
    };

    let tracks = client.search_tracks(query, limit, offset).await?;
    Ok(tracks
        .into_iter()
        .map(|t| Track::Audius(t.into()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{DbConfig, create_pool};

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
            db_pool: pool,
            musicbrainz_client: client,
            audius_client: None,
            podcast_index_client: None,
            cache_ttl_seconds: 3600,
        };

        let _schema = build_schema(app_context);
    }
}
