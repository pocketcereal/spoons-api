//! Podcast database model.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::Value as JsonValue;

use crate::db::schema::podcasts;
use crate::podcast::{Category, Podcast};

/// Database row for podcasts table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = podcasts)]
pub struct PodcastRow {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub artwork_url: Option<String>,
    pub feed_url: String,
    pub language: Option<String>,
    pub categories: JsonValue,
    pub itunes_id: Option<i64>,
    pub episode_count: Option<i32>,
    pub latest_publish_time: Option<DateTime<Utc>>,
    pub trend_score: Option<i32>,
    pub podcast_guid: Option<String>,
    pub cached_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insertable row for podcasts table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = podcasts)]
pub struct NewPodcastRow {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub artwork_url: Option<String>,
    pub feed_url: String,
    pub language: Option<String>,
    pub categories: JsonValue,
    pub itunes_id: Option<i64>,
    pub episode_count: Option<i32>,
    pub latest_publish_time: Option<DateTime<Utc>>,
    pub trend_score: Option<i32>,
    pub podcast_guid: Option<String>,
}

impl From<&Podcast> for NewPodcastRow {
    fn from(podcast: &Podcast) -> Self {
        let categories =
            serde_json::to_value(&podcast.categories).unwrap_or_else(|_| JsonValue::Array(vec![]));

        Self {
            id: podcast.id,
            title: podcast.title.clone(),
            author: podcast.author.clone(),
            description: podcast.description.clone(),
            artwork_url: podcast.artwork_url.clone(),
            feed_url: podcast.feed_url.clone(),
            language: podcast.language.clone(),
            categories,
            itunes_id: podcast.itunes_id,
            episode_count: podcast.episode_count,
            latest_publish_time: podcast.latest_publish_time,
            trend_score: podcast.trend_score,
            podcast_guid: podcast.podcast_guid.clone(),
        }
    }
}

impl From<PodcastRow> for Podcast {
    fn from(row: PodcastRow) -> Self {
        let categories =
            serde_json::from_value::<Vec<Category>>(row.categories).unwrap_or_default();

        Self {
            id: row.id,
            title: row.title,
            author: row.author,
            description: row.description,
            artwork_url: row.artwork_url,
            feed_url: row.feed_url,
            language: row.language,
            categories,
            itunes_id: row.itunes_id,
            episode_count: row.episode_count,
            latest_publish_time: row.latest_publish_time,
            trend_score: row.trend_score,
            podcast_guid: row.podcast_guid,
        }
    }
}
