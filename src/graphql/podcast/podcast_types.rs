//! GraphQL podcast types with source-agnostic interfaces.
//!
//! This module defines GraphQL interfaces that abstract over multiple podcast data sources,
//! allowing mixed search results from PodcastIndex and future sources.

use async_graphql::{Enum, Interface, Object, SimpleObject};
use chrono::{DateTime, Utc};

use crate::podcast::{
    Category as DomainCategory, Podcast as DomainPodcast, PodcastSource as DomainPodcastSource,
};

// ============================================================================
// PodcastSource Enum
// ============================================================================

/// Sources for podcast data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum)]
pub enum PodcastSource {
    /// PodcastIndex podcast directory.
    PodcastIndex,
}

impl From<DomainPodcastSource> for PodcastSource {
    fn from(source: DomainPodcastSource) -> Self {
        match source {
            DomainPodcastSource::PodcastIndex => PodcastSource::PodcastIndex,
        }
    }
}

// ============================================================================
// Category Type
// ============================================================================

/// GraphQL Category type.
#[derive(Debug, Clone, SimpleObject)]
pub struct Category {
    /// Category ID.
    pub id: i32,
    /// Category name.
    pub name: String,
}

impl From<DomainCategory> for Category {
    fn from(cat: DomainCategory) -> Self {
        Self {
            id: cat.id,
            name: cat.name,
        }
    }
}

// ============================================================================
// Podcast Interface and Implementations
// ============================================================================

/// Podcast interface - common fields across all podcast sources.
///
/// Concrete implementations:
/// - `PodcastIndexPodcast` - Podcast from PodcastIndex
#[allow(clippy::duplicated_attributes)] // False positive: each field() is a distinct interface field
#[derive(Interface)]
#[graphql(
    field(
        name = "id",
        ty = "String",
        desc = "Unique identifier (prefixed with source)"
    ),
    field(name = "title", ty = "String", desc = "Podcast title"),
    field(
        name = "source",
        ty = "PodcastSource",
        desc = "Data source this record came from"
    ),
    field(name = "source_id", ty = "String", desc = "ID in the source system"),
    field(name = "author", ty = "Option<String>", desc = "Podcast author"),
    field(
        name = "description",
        ty = "Option<String>",
        desc = "Podcast description"
    ),
    field(name = "artwork_url", ty = "Option<String>", desc = "Artwork URL"),
    field(name = "feed_url", ty = "String", desc = "RSS feed URL"),
    field(name = "language", ty = "Option<String>", desc = "Language code"),
    field(name = "categories", ty = "Vec<Category>", desc = "Podcast categories"),
    field(
        name = "episode_count",
        ty = "Option<i32>",
        desc = "Number of episodes"
    ),
    field(
        name = "latest_publish_time",
        ty = "Option<DateTime<Utc>>",
        desc = "Latest episode publish time"
    )
)]
pub enum Podcast {
    PodcastIndex(PodcastIndexPodcast),
}

/// PodcastIndex-specific podcast implementation.
#[derive(Debug, Clone)]
pub struct PodcastIndexPodcast {
    /// Internal domain podcast.
    pub inner: DomainPodcast,
}

#[Object]
impl PodcastIndexPodcast {
    // Interface fields

    /// Internal ID (prefixed with source).
    async fn id(&self) -> String {
        DomainPodcastSource::PodcastIndex.format_id(self.inner.id)
    }

    /// Podcast title.
    async fn title(&self) -> &str {
        &self.inner.title
    }

    /// Data source.
    async fn source(&self) -> PodcastSource {
        PodcastSource::PodcastIndex
    }

    /// PodcastIndex ID.
    async fn source_id(&self) -> String {
        self.inner.id.to_string()
    }

    /// Podcast author.
    async fn author(&self) -> Option<String> {
        self.inner.author.clone()
    }

    /// Podcast description.
    async fn description(&self) -> Option<String> {
        self.inner.description.clone()
    }

    /// Artwork URL.
    async fn artwork_url(&self) -> Option<String> {
        self.inner.artwork_url.clone()
    }

    /// RSS feed URL.
    async fn feed_url(&self) -> &str {
        &self.inner.feed_url
    }

    /// Language code.
    async fn language(&self) -> Option<String> {
        self.inner.language.clone()
    }

    /// Podcast categories.
    async fn categories(&self) -> Vec<Category> {
        self.inner
            .categories
            .iter()
            .map(|c| Category::from(c.clone()))
            .collect()
    }

    /// Number of episodes.
    async fn episode_count(&self) -> Option<i32> {
        self.inner.episode_count
    }

    /// Latest episode publish time.
    async fn latest_publish_time(&self) -> Option<DateTime<Utc>> {
        self.inner.latest_publish_time
    }

    // PodcastIndex-specific fields

    /// iTunes ID.
    async fn itunes_id(&self) -> Option<i64> {
        self.inner.itunes_id
    }

    /// Trend score.
    async fn trend_score(&self) -> Option<i32> {
        self.inner.trend_score
    }

    /// Podcast GUID.
    async fn podcast_guid(&self) -> Option<&str> {
        self.inner.podcast_guid.as_deref()
    }
}

// ============================================================================
// Conversion Implementations
// ============================================================================

impl From<DomainPodcast> for Podcast {
    fn from(p: DomainPodcast) -> Self {
        Podcast::PodcastIndex(PodcastIndexPodcast { inner: p })
    }
}

impl From<DomainPodcast> for PodcastIndexPodcast {
    fn from(p: DomainPodcast) -> Self {
        PodcastIndexPodcast { inner: p }
    }
}
