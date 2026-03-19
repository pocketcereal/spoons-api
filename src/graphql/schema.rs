use std::sync::Arc;

use async_graphql::{
    Context, EmptyMutation, EmptySubscription, ErrorExtensions, MergedObject, Object, Schema,
};

use crate::domain::{AudiobookProvider, DataSource, MusicProvider, PodcastProvider};
use crate::error::AppError;
use crate::sources::{fan_out_search, SOURCE_TIMEOUT};

use super::audiobook::AudiobookQuery;
use super::podcast::PodcastQuery;
use super::types::{Artist, Track};
use super::unified::UnifiedQuery;

type GqlResult<T> = std::result::Result<T, async_graphql::Error>;

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub struct AppContext {
    pub music_providers: Vec<Arc<dyn MusicProvider>>,
    pub podcast_providers: Vec<Arc<dyn PodcastProvider>>,
    pub audiobook_providers: Vec<Arc<dyn AudiobookProvider>>,
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

pub(crate) fn filter_music_providers(
    providers: &[Arc<dyn MusicProvider>],
    sources: Option<&[DataSource]>,
) -> Vec<Arc<dyn MusicProvider>> {
    match sources {
        Some(allowed) => providers
            .iter()
            .filter(|p| allowed.contains(&p.source_id()))
            .cloned()
            .collect(),
        None => providers.to_vec(),
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
        sources: Option<Vec<DataSource>>,
        #[graphql(default = 25)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> GqlResult<Vec<Artist>> {
        let query = validate_query(&query)?;
        let limit = clamp_limit(limit);
        let offset = clamp_offset(offset);
        let app_ctx = get_app_context(ctx)?;
        let providers = filter_music_providers(&app_ctx.music_providers, sources.as_deref());
        Ok(fan_out_search(&providers, SOURCE_TIMEOUT, |p| {
            let q = query.clone();
            async move { p.search_artists(&q, limit, offset).await }
        })
        .await)
    }

    async fn artist(
        &self,
        ctx: &Context<'_>,
        id: String,
        source: DataSource,
    ) -> GqlResult<Option<Artist>> {
        let id = validate_id(&id)?;
        let app_ctx = get_app_context(ctx)?;
        let provider = app_ctx
            .music_providers
            .iter()
            .find(|p| p.source_id() == source)
            .ok_or_else(|| {
                AppError::FeatureDisabled(format!("{:?} is not configured", source)).extend()
            })?;
        provider.get_artist(&id).await.map_err(|e| e.extend())
    }

    async fn search_tracks(
        &self,
        ctx: &Context<'_>,
        query: String,
        sources: Option<Vec<DataSource>>,
        #[graphql(default = 25)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> GqlResult<Vec<Track>> {
        let query = validate_query(&query)?;
        let limit = clamp_limit(limit);
        let offset = clamp_offset(offset);
        let app_ctx = get_app_context(ctx)?;
        let providers = filter_music_providers(&app_ctx.music_providers, sources.as_deref());
        Ok(fan_out_search(&providers, SOURCE_TIMEOUT, |p| {
            let q = query.clone();
            async move { p.search_tracks(&q, limit, offset).await }
        })
        .await)
    }

    async fn random_tracks(
        &self,
        ctx: &Context<'_>,
        sources: Option<Vec<DataSource>>,
        #[graphql(default = 10)] limit: i32,
    ) -> GqlResult<Vec<Track>> {
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let providers = filter_music_providers(&app_ctx.music_providers, sources.as_deref());
        Ok(
            fan_out_search(&providers, SOURCE_TIMEOUT, |p| async move {
                p.random_tracks(limit).await
            })
            .await,
        )
    }

    async fn random_artists(
        &self,
        ctx: &Context<'_>,
        sources: Option<Vec<DataSource>>,
        #[graphql(default = 10)] limit: i32,
    ) -> GqlResult<Vec<Artist>> {
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let providers = filter_music_providers(&app_ctx.music_providers, sources.as_deref());
        Ok(
            fan_out_search(&providers, SOURCE_TIMEOUT, |p| async move {
                p.random_artists(limit).await
            })
            .await,
        )
    }

    async fn trending_tracks(
        &self,
        ctx: &Context<'_>,
        sources: Option<Vec<DataSource>>,
        #[graphql(default = 20)] limit: i32,
    ) -> GqlResult<Vec<Track>> {
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let providers = filter_music_providers(&app_ctx.music_providers, sources.as_deref());
        Ok(
            fan_out_search(&providers, SOURCE_TIMEOUT, |p| async move {
                p.trending_tracks(limit).await
            })
            .await,
        )
    }

    async fn track(
        &self,
        ctx: &Context<'_>,
        id: String,
        source: DataSource,
    ) -> GqlResult<Option<Track>> {
        let id = validate_id(&id)?;
        let app_ctx = get_app_context(ctx)?;
        let provider = app_ctx
            .music_providers
            .iter()
            .find(|p| p.source_id() == source)
            .ok_or_else(|| {
                AppError::FeatureDisabled(format!("{:?} is not configured", source)).extend()
            })?;
        provider.get_track(&id).await.map_err(|e| e.extend())
    }
}

#[derive(MergedObject, Default)]
pub struct QueryRoot(MusicQuery, PodcastQuery, AudiobookQuery, UnifiedQuery);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_builds() {
        let app_context = AppContext {
            music_providers: vec![],
            podcast_providers: vec![],
            audiobook_providers: vec![],
        };
        let _schema = build_schema(app_context);
    }
}
