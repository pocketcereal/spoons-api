//! GraphQL query resolvers for podcast operations.

use async_graphql::{Context, Object, Result};
use std::sync::Arc;

use crate::error::AppError;
use crate::graphql::AppContext;
use crate::podcast::PodcastSource as DomainPodcastSource;

use super::{Category, Episode, Podcast};

/// Podcast query root.
#[derive(Default)]
pub struct PodcastQuery;

#[Object]
impl PodcastQuery {
    /// Search podcasts by term.
    ///
    /// # Arguments
    /// * `query` - Search term
    /// * `limit` - Maximum number of results (default: 20)
    /// * `offset` - Result offset for pagination (default: 0)
    ///
    /// # Returns
    /// List of matching podcasts
    async fn search_podcasts(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 20)] limit: i32,
        #[graphql(default = 0)] _offset: i32,
    ) -> Result<Vec<Podcast>> {
        let app_ctx = ctx.data::<Arc<AppContext>>()?;

        let client = app_ctx
            .podcast_index_client
            .as_ref()
            .ok_or_else(|| AppError::Server("PodcastIndex not configured".to_string()))?;

        let results = client
            .search_podcasts(&query, limit)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(results.into_iter().map(Podcast::from).collect())
    }

    /// Get trending podcasts.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of results (default: 20)
    /// * `categories` - Optional category IDs to filter by
    ///
    /// # Returns
    /// List of trending podcasts
    async fn trending_podcasts(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20)] limit: i32,
        categories: Option<Vec<i32>>,
    ) -> Result<Vec<Podcast>> {
        let app_ctx = ctx.data::<Arc<AppContext>>()?;

        let client = app_ctx
            .podcast_index_client
            .as_ref()
            .ok_or_else(|| AppError::Server("PodcastIndex not configured".to_string()))?;

        let category_refs: Option<Vec<i32>> = categories;
        let results = client
            .trending(limit, category_refs.as_deref())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(results.into_iter().map(Podcast::from).collect())
    }

    /// Get a podcast by ID.
    ///
    /// # Arguments
    /// * `id` - Podcast ID in the format "podcastindex:12345"
    ///
    /// # Returns
    /// The podcast if found, None otherwise
    async fn podcast(&self, ctx: &Context<'_>, id: String) -> Result<Option<Podcast>> {
        let app_ctx = ctx.data::<Arc<AppContext>>()?;

        // Parse the prefixed ID
        let (source, feed_id) = DomainPodcastSource::parse_id(&id).ok_or_else(|| {
            async_graphql::Error::new(format!("Invalid podcast ID format: {}", id))
        })?;

        match source {
            DomainPodcastSource::PodcastIndex => {
                let client = app_ctx
                    .podcast_index_client
                    .as_ref()
                    .ok_or_else(|| AppError::Server("PodcastIndex not configured".to_string()))?;

                let podcast = client
                    .get_podcast(feed_id)
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?;

                Ok(Some(Podcast::from(podcast)))
            }
        }
    }

    /// Get all podcast categories.
    ///
    /// # Returns
    /// List of all available categories
    async fn podcast_categories(&self, ctx: &Context<'_>) -> Result<Vec<Category>> {
        let app_ctx = ctx.data::<Arc<AppContext>>()?;

        let client = app_ctx
            .podcast_index_client
            .as_ref()
            .ok_or_else(|| AppError::Server("PodcastIndex not configured".to_string()))?;

        let categories = client
            .categories()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(categories.into_iter().map(Category::from).collect())
    }

    /// Get episodes for a podcast.
    ///
    /// # Arguments
    /// * `podcast_id` - Podcast ID in the format "podcastindex:12345"
    /// * `limit` - Maximum number of episodes to return (default: 20)
    ///
    /// # Returns
    /// List of episodes for the podcast
    async fn episodes(
        &self,
        ctx: &Context<'_>,
        podcast_id: String,
        #[graphql(default = 20)] limit: i32,
    ) -> Result<Vec<Episode>> {
        let app_ctx = ctx.data::<Arc<AppContext>>()?;

        // Parse the prefixed podcast ID
        let (source, feed_id) = DomainPodcastSource::parse_id(&podcast_id).ok_or_else(|| {
            async_graphql::Error::new(format!("Invalid podcast ID format: {}", podcast_id))
        })?;

        match source {
            DomainPodcastSource::PodcastIndex => {
                let client = app_ctx
                    .podcast_index_client
                    .as_ref()
                    .ok_or_else(|| AppError::Server("PodcastIndex not configured".to_string()))?;

                let episodes = client
                    .get_episodes(feed_id, limit)
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?;

                Ok(episodes.into_iter().map(Episode::from).collect())
            }
        }
    }

    /// Get a single episode by ID.
    ///
    /// # Arguments
    /// * `id` - Episode ID in the format "podcastindex:98765"
    ///
    /// # Returns
    /// The episode if found, None otherwise
    async fn episode(&self, ctx: &Context<'_>, id: String) -> Result<Option<Episode>> {
        let app_ctx = ctx.data::<Arc<AppContext>>()?;

        // Parse the prefixed ID
        let (source, episode_id) = DomainPodcastSource::parse_id(&id).ok_or_else(|| {
            async_graphql::Error::new(format!("Invalid episode ID format: {}", id))
        })?;

        match source {
            DomainPodcastSource::PodcastIndex => {
                let client = app_ctx
                    .podcast_index_client
                    .as_ref()
                    .ok_or_else(|| AppError::Server("PodcastIndex not configured".to_string()))?;

                let episode = client
                    .get_episode(episode_id)
                    .await
                    .map_err(|e| async_graphql::Error::new(e.to_string()))?;

                Ok(Some(Episode::from(episode)))
            }
        }
    }

    /// Get random episodes for discovery.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of episodes to return (default: 10)
    /// * `language` - Optional language code filter (e.g., "en")
    /// * `categories` - Optional category IDs to filter by
    ///
    /// # Returns
    /// List of random episodes
    async fn random_episodes(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i32,
        language: Option<String>,
        categories: Option<Vec<i32>>,
    ) -> Result<Vec<Episode>> {
        let app_ctx = ctx.data::<Arc<AppContext>>()?;

        let client = app_ctx
            .podcast_index_client
            .as_ref()
            .ok_or_else(|| AppError::Server("PodcastIndex not configured".to_string()))?;

        let category_refs: Option<Vec<i32>> = categories;
        let episodes = client
            .random_episodes(limit, language.as_deref(), category_refs.as_deref())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(episodes.into_iter().map(Episode::from).collect())
    }
}
