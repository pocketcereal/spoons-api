//! GraphQL schema definition and query handlers.

use async_graphql::{Context, EmptyMutation, EmptySubscription, ErrorExtensions, Object, Schema};
use std::sync::Arc;

use crate::audius::AudiusClient;
use crate::db::{DbPool, MusicRepository};
use crate::domain::DataSource;
use crate::error::AppError;
use crate::musicbrainz::MusicBrainzClient;

use super::types::{Artist, Track};

/// Result type alias for GraphQL resolvers that converts AppError to async_graphql::Error
type GqlResult<T> = std::result::Result<T, async_graphql::Error>;

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(Clone)]
pub struct AppContext {
    pub db_pool: DbPool,
    pub musicbrainz_client: MusicBrainzClient,
    pub audius_client: Option<AudiusClient>,
    pub cache_ttl_seconds: i64,
}

pub fn build_schema(app_context: AppContext) -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(Arc::new(app_context))
        .finish()
}

fn get_app_context<'a>(ctx: &'a Context<'_>) -> Result<&'a Arc<AppContext>, async_graphql::Error> {
    ctx.data::<Arc<AppContext>>().map_err(|_| {
        AppError::Internal(anyhow::anyhow!("Application context not configured")).extend()
    })
}

/// Maximum allowed limit for search queries.
const MAX_SEARCH_LIMIT: i32 = 100;

/// Maximum allowed offset for search queries.
const MAX_SEARCH_OFFSET: i32 = 10000;

/// Clamps the limit to the maximum allowed value.
fn clamp_limit(limit: i32) -> i32 {
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
        .ok_or_else(|| AppError::Server("Audius client not available".to_string()).extend())
}

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
            let (mb_results, audius_results) = tokio::join!(mb_search, audius_search);

            let mut combined = Vec::new();

            match mb_results {
                Ok(items) => combined.extend(items),
                Err(e) => tracing::warn!(error = %e, "MusicBrainz {} search failed", entity_name),
            }

            match audius_results {
                Ok(items) => combined.extend(items),
                Err(e) => tracing::warn!(error = %e, "Audius {} search failed", entity_name),
            }

            Ok(combined)
        }
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
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
            cache_ttl_seconds: 3600,
        };

        let _schema = build_schema(app_context);
    }
}
