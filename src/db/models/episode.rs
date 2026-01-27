//! Episode database model.

use chrono::{DateTime, Utc};
use diesel::prelude::*;

use crate::db::schema::episodes;
use crate::podcast::Episode;

/// Database row for episodes table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = episodes)]
pub struct EpisodeRow {
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
    pub cached_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insertable row for episodes table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = episodes)]
pub struct NewEpisodeRow {
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

impl From<&Episode> for NewEpisodeRow {
    fn from(episode: &Episode) -> Self {
        Self {
            id: episode.id,
            podcast_id: episode.podcast_id,
            title: episode.title.clone(),
            description: episode.description.clone(),
            audio_url: episode.audio_url.clone(),
            audio_type: episode.audio_type.clone(),
            audio_length: episode.audio_length,
            duration_seconds: episode.duration_seconds,
            published_at: episode.published_at,
            episode_number: episode.episode_number,
            season_number: episode.season_number,
            episode_type: episode.episode_type.clone(),
            image_url: episode.image_url.clone(),
            explicit: episode.explicit,
        }
    }
}

impl From<EpisodeRow> for Episode {
    fn from(row: EpisodeRow) -> Self {
        Self {
            id: row.id,
            podcast_id: row.podcast_id,
            title: row.title,
            description: row.description,
            audio_url: row.audio_url,
            audio_type: row.audio_type,
            audio_length: row.audio_length,
            duration_seconds: row.duration_seconds,
            published_at: row.published_at,
            episode_number: row.episode_number,
            season_number: row.season_number,
            episode_type: row.episode_type,
            image_url: row.image_url,
            explicit: row.explicit,
        }
    }
}
