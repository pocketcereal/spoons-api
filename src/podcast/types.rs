use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A podcast show (source-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Podcast {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub artwork_url: Option<String>,
    pub feed_url: String,
    pub language: Option<String>,
    pub categories: Vec<Category>,
    pub episode_count: Option<i32>,
    pub latest_publish_time: Option<DateTime<Utc>>,
    // Source-specific fields that may be present
    pub itunes_id: Option<i64>,
    pub trend_score: Option<i32>,
    pub podcast_guid: Option<String>,
}

/// A podcast episode (source-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: i64,
    pub podcast_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub audio_url: String,
    pub audio_type: Option<String>,
    pub audio_length: Option<i64>,
    pub duration_seconds: Option<i32>,
    pub published_at: Option<DateTime<Utc>>,
    pub episode_number: Option<i32>,
    pub season_number: Option<i32>,
    pub episode_type: Option<String>,
    pub image_url: Option<String>,
    pub explicit: Option<bool>,
}

/// A podcast category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
}
