//! PodcastIndex API response types.
//!
//! These types model the JSON structure of PodcastIndex API responses.
//! Only fields actively used by application code are included; serde
//! silently ignores any extra fields in the API response.

use serde::Deserialize;

/// Response wrapper for PodcastIndex list endpoints (search, trending, categories).
///
/// These endpoints return results under the `feeds` key.
#[derive(Debug, Clone, Deserialize)]
pub struct PodcastIndexListResponse<T> {
    pub feeds: Option<Vec<T>>,
}

/// Response from the Podcast Index podcast by ID endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct PodcastIndexPodcastResponse {
    pub feed: PodcastFeed,
}

/// Response from the Podcast Index episode-by-id endpoint.
/// Returns a single episode under the `episode` key.
#[derive(Debug, Clone, Deserialize)]
pub struct PodcastIndexEpisodeByIdResponse {
    pub episode: PodcastEpisode,
}

/// Response from the Podcast Index episodes-by-feed endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct PodcastIndexEpisodesResponse {
    pub items: Vec<PodcastEpisode>,
}

/// Response from the Podcast Index random episodes endpoint.
/// Returns episodes under the `episodes` key.
#[derive(Debug, Clone, Deserialize)]
pub struct PodcastIndexRandomEpisodesResponse {
    pub episodes: Vec<PodcastEpisode>,
}

/// A podcast feed from the Podcast Index API.
#[derive(Debug, Clone, Deserialize)]
pub struct PodcastFeed {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub artwork: Option<String>,
    pub url: String,
    #[serde(rename = "itunesId")]
    pub itunes_id: Option<i64>,
    pub language: String,
    pub categories: Option<serde_json::Value>,
    #[serde(rename = "newestItemPublishTime")]
    pub newest_item_publish_time: Option<i64>,
    #[serde(rename = "trendScore")]
    pub trend_score: Option<i32>,
}

/// An episode from the Podcast Index API.
#[derive(Debug, Clone, Deserialize)]
pub struct PodcastEpisode {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub guid: String,
    pub link: Option<String>,
    #[serde(rename = "datePublished", default)]
    pub date_published: i64,
    #[serde(rename = "datePublishedPretty")]
    pub date_published_pretty: Option<String>,
    #[serde(rename = "dateCrawled", default)]
    pub date_crawled: i64,
    #[serde(rename = "enclosureUrl", default)]
    pub enclosure_url: String,
    #[serde(rename = "enclosureType", default)]
    pub enclosure_type: String,
    #[serde(rename = "enclosureLength", default)]
    pub enclosure_length: i64,
    pub duration: Option<i32>,
    #[serde(default)]
    pub explicit: i32,
    pub episode: Option<i32>,
    pub season: Option<i32>,
    #[serde(rename = "episodeType")]
    pub episode_type: Option<String>,
    pub image: Option<String>,
    #[serde(rename = "feedId")]
    pub feed_id: i64,
    #[serde(rename = "feedTitle")]
    pub feed_title: Option<String>,
    #[serde(rename = "feedImage")]
    pub feed_image: Option<String>,
    #[serde(rename = "feedItunesId")]
    pub feed_itunes_id: Option<i64>,
    #[serde(rename = "feedLanguage")]
    pub feed_language: Option<String>,
    #[serde(rename = "feedUrl")]
    pub feed_url: Option<String>,
    #[serde(rename = "feedDead")]
    pub feed_dead: Option<i32>,
    #[serde(rename = "feedDuplicateOf")]
    pub feed_duplicate_of: Option<i64>,
    #[serde(rename = "chaptersUrl")]
    pub chapters_url: Option<String>,
    #[serde(rename = "transcriptUrl")]
    pub transcript_url: Option<String>,
    #[serde(rename = "podcastGuid")]
    pub podcast_guid: Option<String>,
}

/// Category information for podcasts.
#[derive(Debug, Clone, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
}
