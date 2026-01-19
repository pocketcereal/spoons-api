//! GraphQL schema definition and query handlers.

use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};
use std::sync::Arc;

use crate::db::{DbPool, MusicRepository};
use crate::error::AppError;
use crate::musicbrainz::{Artist, MusicBrainzClient, Recording, Release, ReleaseGroup};

/// Application GraphQL schema type.
pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

/// Application context for GraphQL resolvers.
#[derive(Clone)]
pub struct AppContext {
    /// Database connection pool.
    pub db_pool: DbPool,
    /// MusicBrainz API client.
    pub musicbrainz_client: MusicBrainzClient,
    /// Cache TTL in seconds.
    pub cache_ttl_seconds: i64,
}

/// Build the GraphQL schema.
pub fn build_schema(app_context: AppContext) -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(Arc::new(app_context))
        .finish()
}

/// Root query object.
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get the API version.
    async fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    /// Search for artists by name.
    async fn search_artists(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 25)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> Result<Vec<Artist>, AppError> {
        let app_ctx = ctx.data::<Arc<AppContext>>().map_err(|_| {
            AppError::Internal(anyhow::anyhow!("Application context not configured"))
        })?;

        MusicRepository::search_artists(
            &app_ctx.db_pool,
            &app_ctx.musicbrainz_client,
            &query,
            limit,
            offset,
            app_ctx.cache_ttl_seconds,
        )
        .await
    }

    /// Get an artist by MusicBrainz ID.
    async fn artist(&self, ctx: &Context<'_>, id: String) -> Result<Artist, AppError> {
        let app_ctx = ctx.data::<Arc<AppContext>>().map_err(|_| {
            AppError::Internal(anyhow::anyhow!("Application context not configured"))
        })?;

        MusicRepository::get_artist(
            &app_ctx.db_pool,
            &app_ctx.musicbrainz_client,
            &id,
            app_ctx.cache_ttl_seconds,
        )
        .await
    }

    /// Search for releases by name.
    async fn search_releases(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 25)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> Result<Vec<Release>, AppError> {
        let app_ctx = ctx.data::<Arc<AppContext>>().map_err(|_| {
            AppError::Internal(anyhow::anyhow!("Application context not configured"))
        })?;

        MusicRepository::search_releases(
            &app_ctx.db_pool,
            &app_ctx.musicbrainz_client,
            &query,
            limit,
            offset,
            app_ctx.cache_ttl_seconds,
        )
        .await
    }

    /// Get a release by MusicBrainz ID.
    async fn release(&self, ctx: &Context<'_>, id: String) -> Result<Release, AppError> {
        let app_ctx = ctx.data::<Arc<AppContext>>().map_err(|_| {
            AppError::Internal(anyhow::anyhow!("Application context not configured"))
        })?;

        MusicRepository::get_release(
            &app_ctx.db_pool,
            &app_ctx.musicbrainz_client,
            &id,
            app_ctx.cache_ttl_seconds,
        )
        .await
    }

    /// Search for recordings by name.
    async fn search_recordings(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 25)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> Result<Vec<Recording>, AppError> {
        let app_ctx = ctx.data::<Arc<AppContext>>().map_err(|_| {
            AppError::Internal(anyhow::anyhow!("Application context not configured"))
        })?;

        MusicRepository::search_recordings(
            &app_ctx.db_pool,
            &app_ctx.musicbrainz_client,
            &query,
            limit,
            offset,
            app_ctx.cache_ttl_seconds,
        )
        .await
    }

    /// Get a recording by MusicBrainz ID.
    async fn recording(&self, ctx: &Context<'_>, id: String) -> Result<Recording, AppError> {
        let app_ctx = ctx.data::<Arc<AppContext>>().map_err(|_| {
            AppError::Internal(anyhow::anyhow!("Application context not configured"))
        })?;

        MusicRepository::get_recording(
            &app_ctx.db_pool,
            &app_ctx.musicbrainz_client,
            &id,
            app_ctx.cache_ttl_seconds,
        )
        .await
    }

    /// Search for release groups by name.
    async fn search_release_groups(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 25)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> Result<Vec<ReleaseGroup>, AppError> {
        let app_ctx = ctx.data::<Arc<AppContext>>().map_err(|_| {
            AppError::Internal(anyhow::anyhow!("Application context not configured"))
        })?;

        MusicRepository::search_release_groups(
            &app_ctx.db_pool,
            &app_ctx.musicbrainz_client,
            &query,
            limit,
            offset,
            app_ctx.cache_ttl_seconds,
        )
        .await
    }

    /// Get a release group by MusicBrainz ID.
    async fn release_group(&self, ctx: &Context<'_>, id: String) -> Result<ReleaseGroup, AppError> {
        let app_ctx = ctx.data::<Arc<AppContext>>().map_err(|_| {
            AppError::Internal(anyhow::anyhow!("Application context not configured"))
        })?;

        MusicRepository::get_release_group(
            &app_ctx.db_pool,
            &app_ctx.musicbrainz_client,
            &id,
            app_ctx.cache_ttl_seconds,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{DbConfig, create_pool};

    #[test]
    fn test_schema_builds() {
        // Create a mock/test context - note this test will fail without a real DB
        // In a real scenario, we'd use a test database or mock
        let client = MusicBrainzClient::new("https://musicbrainz.org/ws/2").unwrap();

        // Skip test if no DATABASE_URL - this is expected in CI without DB
        let db_url = match std::env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => return, // Skip test if no DB configured
        };

        let db_config = DbConfig {
            url: db_url,
            max_connections: 1,
        };

        let pool = create_pool(&db_config).unwrap();

        let app_context = AppContext {
            db_pool: pool,
            musicbrainz_client: client,
            cache_ttl_seconds: 3600,
        };

        let _schema = build_schema(app_context);
    }
}
