//! GraphQL query resolvers for podcast operations.

use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::error::AppError;
use crate::graphql::{clamp_limit, get_app_context, require_podcast_index_client, validate_query};
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
    ///
    /// # Returns
    /// List of matching podcasts
    async fn search_podcasts(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 20)] limit: i32,
    ) -> Result<Vec<Podcast>> {
        let query = validate_query(&query)?;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let client = require_podcast_index_client(app_ctx)?;

        let results = client
            .search_podcasts(&query, limit)
            .await
            .map_err(|e| e.extend())?;

        Ok(results.into_iter().map(Podcast::from).collect())
    }

    /// Search podcasts by title.
    ///
    /// # Arguments
    /// * `title` - Title to search for
    /// * `limit` - Maximum number of results (default: 20)
    ///
    /// # Returns
    /// List of matching podcasts (title-only search for more precise results)
    async fn search_podcasts_by_title(
        &self,
        ctx: &Context<'_>,
        title: String,
        #[graphql(default = 20)] limit: i32,
    ) -> Result<Vec<Podcast>> {
        let title = validate_query(&title)?;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let client = require_podcast_index_client(app_ctx)?;

        let results = client
            .search_by_title(&title, limit)
            .await
            .map_err(|e| e.extend())?;

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
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let client = require_podcast_index_client(app_ctx)?;

        let results = client
            .trending(limit, categories.as_deref())
            .await
            .map_err(|e| e.extend())?;

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
        let app_ctx = get_app_context(ctx)?;

        // Parse the prefixed ID
        let (source, feed_id) = DomainPodcastSource::parse_id(&id).ok_or_else(|| {
            AppError::InvalidInput(format!("Invalid podcast ID format: {}", id)).extend()
        })?;

        match source {
            DomainPodcastSource::PodcastIndex => {
                let client = require_podcast_index_client(app_ctx)?;

                match client.get_podcast(feed_id).await {
                    Ok(podcast) => Ok(Some(Podcast::from(podcast))),
                    Err(AppError::NotFound(_)) => Ok(None),
                    Err(e) => Err(e.extend()),
                }
            }
        }
    }

    /// Get all podcast categories.
    ///
    /// # Returns
    /// List of all available categories
    async fn podcast_categories(&self, ctx: &Context<'_>) -> Result<Vec<Category>> {
        let app_ctx = get_app_context(ctx)?;
        let client = require_podcast_index_client(app_ctx)?;

        let categories = client.categories().await.map_err(|e| e.extend())?;

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
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;

        // Parse the prefixed podcast ID
        let (source, feed_id) = DomainPodcastSource::parse_id(&podcast_id).ok_or_else(|| {
            AppError::InvalidInput(format!("Invalid podcast ID format: {}", podcast_id)).extend()
        })?;

        match source {
            DomainPodcastSource::PodcastIndex => {
                let client = require_podcast_index_client(app_ctx)?;

                let episodes = client
                    .get_episodes(feed_id, limit)
                    .await
                    .map_err(|e| e.extend())?;

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
        let app_ctx = get_app_context(ctx)?;

        // Parse the prefixed ID
        let (source, episode_id) = DomainPodcastSource::parse_id(&id).ok_or_else(|| {
            AppError::InvalidInput(format!("Invalid episode ID format: {}", id)).extend()
        })?;

        match source {
            DomainPodcastSource::PodcastIndex => {
                let client = require_podcast_index_client(app_ctx)?;

                match client.get_episode(episode_id).await {
                    Ok(episode) => Ok(Some(Episode::from(episode))),
                    Err(AppError::NotFound(_)) => Ok(None),
                    Err(e) => Err(e.extend()),
                }
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
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let client = require_podcast_index_client(app_ctx)?;

        let episodes = client
            .random_episodes(limit, language.as_deref(), categories.as_deref())
            .await
            .map_err(|e| e.extend())?;

        Ok(episodes.into_iter().map(Episode::from).collect())
    }
}
