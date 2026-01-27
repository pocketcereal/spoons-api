//! GraphQL episode types with source-agnostic interfaces.
//!
//! This module defines GraphQL interfaces that abstract over multiple podcast episode data sources,
//! allowing mixed search results from PodcastIndex and future sources.

use async_graphql::{Interface, Object};
use chrono::{DateTime, Utc};

use crate::podcast::{Episode as DomainEpisode, PodcastSource as DomainPodcastSource};

use super::podcast_types::PodcastSource;

// ============================================================================
// Episode Interface and Implementations
// ============================================================================

/// Episode interface - common fields across all podcast episode sources.
///
/// Concrete implementations:
/// - `PodcastIndexEpisode` - Episode from PodcastIndex
#[allow(clippy::duplicated_attributes)] // False positive: each field() is a distinct interface field
#[derive(Interface)]
#[graphql(
    field(
        name = "id",
        ty = "String",
        desc = "Unique identifier (prefixed with source)"
    ),
    field(name = "title", ty = "String", desc = "Episode title"),
    field(
        name = "source",
        ty = "PodcastSource",
        desc = "Data source this record came from"
    ),
    field(name = "source_id", ty = "String", desc = "ID in the source system"),
    field(
        name = "podcast_id",
        ty = "String",
        desc = "Podcast ID (prefixed with source)"
    ),
    field(
        name = "description",
        ty = "Option<String>",
        desc = "Episode description"
    ),
    field(name = "audio_url", ty = "String", desc = "Audio file URL"),
    field(
        name = "duration_seconds",
        ty = "Option<i32>",
        desc = "Duration in seconds"
    ),
    field(
        name = "published_at",
        ty = "Option<DateTime<Utc>>",
        desc = "Publication date"
    ),
    field(name = "episode_number", ty = "Option<i32>", desc = "Episode number"),
    field(name = "season_number", ty = "Option<i32>", desc = "Season number"),
    field(name = "image_url", ty = "Option<String>", desc = "Episode image URL")
)]
pub enum Episode {
    PodcastIndex(PodcastIndexEpisode),
}

/// PodcastIndex-specific episode implementation.
#[derive(Debug, Clone)]
pub struct PodcastIndexEpisode {
    /// Internal domain episode.
    pub inner: DomainEpisode,
}

#[Object]
impl PodcastIndexEpisode {
    // Interface fields

    /// Internal ID (prefixed with source).
    async fn id(&self) -> String {
        DomainPodcastSource::PodcastIndex.format_id(self.inner.id)
    }

    /// Episode title.
    async fn title(&self) -> &str {
        &self.inner.title
    }

    /// Data source.
    async fn source(&self) -> PodcastSource {
        PodcastSource::PodcastIndex
    }

    /// PodcastIndex episode ID.
    async fn source_id(&self) -> String {
        self.inner.id.to_string()
    }

    /// Podcast ID (prefixed with source).
    async fn podcast_id(&self) -> String {
        DomainPodcastSource::PodcastIndex.format_id(self.inner.podcast_id)
    }

    /// Episode description.
    async fn description(&self) -> Option<String> {
        self.inner.description.clone()
    }

    /// Audio file URL.
    async fn audio_url(&self) -> &str {
        &self.inner.audio_url
    }

    /// Duration in seconds.
    async fn duration_seconds(&self) -> Option<i32> {
        self.inner.duration_seconds
    }

    /// Publication date.
    async fn published_at(&self) -> Option<DateTime<Utc>> {
        self.inner.published_at
    }

    /// Episode number.
    async fn episode_number(&self) -> Option<i32> {
        self.inner.episode_number
    }

    /// Season number.
    async fn season_number(&self) -> Option<i32> {
        self.inner.season_number
    }

    /// Episode image URL.
    async fn image_url(&self) -> Option<String> {
        self.inner.image_url.clone()
    }

    // PodcastIndex-specific fields

    /// Audio file MIME type.
    async fn audio_type(&self) -> Option<&str> {
        self.inner.audio_type.as_deref()
    }

    /// Audio file size in bytes.
    async fn audio_length(&self) -> Option<i64> {
        self.inner.audio_length
    }

    /// Episode type (full, trailer, bonus, etc.).
    async fn episode_type(&self) -> Option<&str> {
        self.inner.episode_type.as_deref()
    }

    /// Whether the episode contains explicit content.
    async fn explicit(&self) -> Option<bool> {
        self.inner.explicit
    }
}

// ============================================================================
// Conversion Implementations
// ============================================================================

impl From<DomainEpisode> for Episode {
    fn from(e: DomainEpisode) -> Self {
        Episode::PodcastIndex(PodcastIndexEpisode { inner: e })
    }
}

impl From<DomainEpisode> for PodcastIndexEpisode {
    fn from(e: DomainEpisode) -> Self {
        PodcastIndexEpisode { inner: e }
    }
}
