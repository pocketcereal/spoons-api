use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::error::AppError;
use crate::graphql::{clamp_limit, get_app_context, require_podcast_service, validate_query};
use crate::podcast::PodcastSource as DomainPodcastSource;

use super::podcast_types::PodcastSource;
use super::{Category, Episode, Podcast};
#[derive(Default)]
pub struct PodcastQuery;

#[Object]
impl PodcastQuery {
    /// Search podcasts by term.
    async fn search_podcasts(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 20)] limit: i32,
        source: Option<PodcastSource>,
    ) -> Result<Vec<Podcast>> {
        let _ = source;
        let query = validate_query(&query)?;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let service = require_podcast_service(app_ctx)?;

        let results = service
            .search_podcasts(&query, limit)
            .await
            .map_err(|e| e.extend())?;

        Ok(results.into_iter().map(Podcast::from).collect())
    }

    /// Search podcasts by title.
    async fn search_podcasts_by_title(
        &self,
        ctx: &Context<'_>,
        title: String,
        #[graphql(default = 20)] limit: i32,
        source: Option<PodcastSource>,
    ) -> Result<Vec<Podcast>> {
        let _ = source;
        let title = validate_query(&title)?;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let service = require_podcast_service(app_ctx)?;

        let results = service
            .search_by_title(&title, limit)
            .await
            .map_err(|e| e.extend())?;

        Ok(results.into_iter().map(Podcast::from).collect())
    }

    /// Get trending podcasts.
    async fn trending_podcasts(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20)] limit: i32,
        categories: Option<Vec<i32>>,
        source: Option<PodcastSource>,
    ) -> Result<Vec<Podcast>> {
        let _ = source;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let service = require_podcast_service(app_ctx)?;

        let results = service
            .client()
            .trending(limit, categories.as_deref())
            .await
            .map_err(|e| e.extend())?;

        Ok(results.into_iter().map(Podcast::from).collect())
    }

    /// Get a podcast by ID.
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
                let service = require_podcast_service(app_ctx)?;
                let podcast = service
                    .get_podcast(feed_id)
                    .await
                    .map_err(|e| e.extend())?;
                Ok(podcast.map(Podcast::from))
            }
        }
    }

    /// Get all podcast categories.
    async fn podcast_categories(&self, ctx: &Context<'_>) -> Result<Vec<Category>> {
        let app_ctx = get_app_context(ctx)?;
        let service = require_podcast_service(app_ctx)?;

        let categories = service.client().categories().await.map_err(|e| e.extend())?;

        Ok(categories.into_iter().map(Category::from).collect())
    }

    /// Get episodes for a podcast.
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
                let service = require_podcast_service(app_ctx)?;
                let episodes = service
                    .get_episodes(feed_id, limit)
                    .await
                    .map_err(|e| e.extend())?;

                Ok(episodes.into_iter().map(Episode::from).collect())
            }
        }
    }

    /// Get a single episode by ID.
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
                let service = require_podcast_service(app_ctx)?;
                let episode = service
                    .get_episode(episode_id)
                    .await
                    .map_err(|e| e.extend())?;
                Ok(episode.map(Episode::from))
            }
        }
    }

    /// Get random episodes for discovery.
    async fn random_episodes(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i32,
        language: Option<String>,
        categories: Option<Vec<i32>>,
        source: Option<PodcastSource>,
    ) -> Result<Vec<Episode>> {
        let _ = source;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let service = require_podcast_service(app_ctx)?;

        let episodes = service
            .client()
            .random_episodes(limit, language.as_deref(), categories.as_deref())
            .await
            .map_err(|e| e.extend())?;

        Ok(episodes.into_iter().map(Episode::from).collect())
    }
}
