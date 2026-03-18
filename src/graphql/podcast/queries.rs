use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::domain::PodcastProvider;
use crate::error::AppError;
use crate::graphql::schema::AppContext;
use crate::graphql::{clamp_limit, get_app_context, validate_query};
use crate::podcast::PodcastSource as DomainPodcastSource;

use super::{Category, Episode, Podcast};

fn require_podcast_provider(
    app_ctx: &std::sync::Arc<AppContext>,
) -> Result<&std::sync::Arc<dyn PodcastProvider>> {
    app_ctx.podcast_providers.first().ok_or_else(|| {
        AppError::FeatureDisabled(
            "PodcastIndex is not configured. Set podcast_index in config.yaml.".into(),
        )
        .extend()
    })
}

#[derive(Default)]
pub struct PodcastQuery;

#[Object]
impl PodcastQuery {
    async fn search_podcasts(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 20)] limit: i32,
    ) -> Result<Vec<Podcast>> {
        let query = validate_query(&query)?;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let provider = require_podcast_provider(app_ctx)?;
        provider
            .search_podcasts(&query, limit)
            .await
            .map_err(|e| e.extend())
    }

    async fn search_podcasts_by_title(
        &self,
        ctx: &Context<'_>,
        title: String,
        #[graphql(default = 20)] limit: i32,
    ) -> Result<Vec<Podcast>> {
        let title = validate_query(&title)?;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let provider = require_podcast_provider(app_ctx)?;
        provider
            .search_by_title(&title, limit)
            .await
            .map_err(|e| e.extend())
    }

    async fn trending_podcasts(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20)] limit: i32,
        categories: Option<Vec<i32>>,
    ) -> Result<Vec<Podcast>> {
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let provider = require_podcast_provider(app_ctx)?;
        provider
            .trending(limit, categories.as_deref())
            .await
            .map_err(|e| e.extend())
    }

    async fn podcast(&self, ctx: &Context<'_>, id: String) -> Result<Option<Podcast>> {
        let app_ctx = get_app_context(ctx)?;

        let (source, feed_id) = DomainPodcastSource::parse_id(&id).ok_or_else(|| {
            AppError::InvalidInput(format!(
                "Invalid podcast ID format: {}",
                id.chars().take(50).collect::<String>()
            ))
            .extend()
        })?;

        match source {
            DomainPodcastSource::PodcastIndex => {
                let provider = require_podcast_provider(app_ctx)?;
                provider.get_podcast(feed_id).await.map_err(|e| e.extend())
            }
        }
    }

    async fn podcast_categories(&self, ctx: &Context<'_>) -> Result<Vec<Category>> {
        let app_ctx = get_app_context(ctx)?;
        let provider = require_podcast_provider(app_ctx)?;
        provider.categories().await.map_err(|e| e.extend())
    }

    async fn episodes(
        &self,
        ctx: &Context<'_>,
        podcast_id: String,
        #[graphql(default = 20)] limit: i32,
    ) -> Result<Vec<Episode>> {
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;

        let (source, feed_id) = DomainPodcastSource::parse_id(&podcast_id).ok_or_else(|| {
            AppError::InvalidInput(format!(
                "Invalid podcast ID format: {}",
                podcast_id.chars().take(50).collect::<String>()
            ))
            .extend()
        })?;

        match source {
            DomainPodcastSource::PodcastIndex => {
                let provider = require_podcast_provider(app_ctx)?;
                provider
                    .get_episodes(feed_id, limit)
                    .await
                    .map_err(|e| e.extend())
            }
        }
    }

    async fn episode(&self, ctx: &Context<'_>, id: String) -> Result<Option<Episode>> {
        let app_ctx = get_app_context(ctx)?;

        let (source, episode_id) = DomainPodcastSource::parse_id(&id).ok_or_else(|| {
            AppError::InvalidInput(format!(
                "Invalid episode ID format: {}",
                id.chars().take(50).collect::<String>()
            ))
            .extend()
        })?;

        match source {
            DomainPodcastSource::PodcastIndex => {
                let provider = require_podcast_provider(app_ctx)?;
                provider
                    .get_episode(episode_id)
                    .await
                    .map_err(|e| e.extend())
            }
        }
    }

    async fn random_episodes(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i32,
        language: Option<String>,
        categories: Option<Vec<i32>>,
    ) -> Result<Vec<Episode>> {
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let provider = require_podcast_provider(app_ctx)?;
        provider
            .random_episodes(limit, language.as_deref(), categories.as_deref())
            .await
            .map_err(|e| e.extend())
    }
}
