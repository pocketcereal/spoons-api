//! PodcastIndex API response types.

use serde::{Deserialize, Serialize};

/// Generic response wrapper for PodcastIndex API responses.
///
/// Different endpoints use different field names for the data payload:
/// - `feeds` for search results
/// - `feed` for single podcast
/// - `items` for episodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastIndexResponse<T> {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feeds: Option<Vec<T>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<T>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
}

/// Response from the Podcast Index podcast by ID endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastIndexPodcastResponse {
    pub status: String,
    pub feed: PodcastFeed,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response from the Podcast Index episodes endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastIndexEpisodesResponse {
    pub status: String,
    pub items: Vec<PodcastEpisode>,
    pub count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

/// A podcast feed from the Podcast Index API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastFeed {
    pub id: i64,
    pub title: String,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artwork: Option<String>,
    pub url: String,
    #[serde(rename = "itunesId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itunes_id: Option<i64>,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<serde_json::Value>,
    #[serde(rename = "newestItemPublishTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newest_item_publish_time: Option<i64>,
    #[serde(rename = "trendScore")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trend_score: Option<i32>,
}

/// An episode from the Podcast Index API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastEpisode {
    pub id: i64,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub guid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(rename = "datePublished")]
    pub date_published: i64,
    #[serde(rename = "datePublishedPretty")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_published_pretty: Option<String>,
    #[serde(rename = "dateCrawled")]
    pub date_crawled: i64,
    #[serde(rename = "enclosureUrl")]
    pub enclosure_url: String,
    #[serde(rename = "enclosureType")]
    pub enclosure_type: String,
    #[serde(rename = "enclosureLength")]
    pub enclosure_length: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    pub explicit: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub season: Option<i32>,
    #[serde(rename = "episodeType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(rename = "feedId")]
    pub feed_id: i64,
    #[serde(rename = "feedTitle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_title: Option<String>,
    #[serde(rename = "feedImage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_image: Option<String>,
    #[serde(rename = "feedItunesId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_itunes_id: Option<i64>,
    #[serde(rename = "feedLanguage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_language: Option<String>,
    #[serde(rename = "feedUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_url: Option<String>,
    #[serde(rename = "feedDead")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_dead: Option<i32>,
    #[serde(rename = "feedDuplicateOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_duplicate_of: Option<i64>,
    #[serde(rename = "chaptersUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapters_url: Option<String>,
    #[serde(rename = "transcriptUrl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_url: Option<String>,
    #[serde(rename = "podcastGuid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub podcast_guid: Option<String>,
}

/// Category information for podcasts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
}
