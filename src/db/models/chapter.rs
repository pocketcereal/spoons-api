use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::Value as JsonValue;

use crate::audiobook::Chapter;
use crate::db::schema::chapters;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = chapters)]
pub struct ChapterRow {
    pub id: i64,
    pub audiobook_id: i64,
    pub title: String,
    pub section_number: i32,
    pub duration: Option<String>,
    pub duration_seconds: Option<i32>,
    pub listen_url: String,
    pub language: Option<String>,
    pub readers: JsonValue,
    pub cached_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = chapters)]
pub struct NewChapterRow {
    pub id: i64,
    pub audiobook_id: i64,
    pub title: String,
    pub section_number: i32,
    pub duration: Option<String>,
    pub duration_seconds: Option<i32>,
    pub listen_url: String,
    pub language: Option<String>,
    pub readers: JsonValue,
}

impl From<&Chapter> for NewChapterRow {
    fn from(chapter: &Chapter) -> Self {
        let readers =
            serde_json::to_value(&chapter.readers).unwrap_or_else(|_| JsonValue::Array(vec![]));

        Self {
            id: chapter.id,
            audiobook_id: chapter.audiobook_id,
            title: chapter.title.clone(),
            section_number: chapter.section_number,
            duration: chapter.duration.clone(),
            duration_seconds: chapter.duration_seconds,
            listen_url: chapter.listen_url.clone(),
            language: chapter.language.clone(),
            readers,
        }
    }
}

impl From<ChapterRow> for Chapter {
    fn from(row: ChapterRow) -> Self {
        let readers = serde_json::from_value::<Vec<String>>(row.readers).unwrap_or_default();

        Self {
            id: row.id,
            audiobook_id: row.audiobook_id,
            title: row.title,
            section_number: row.section_number,
            duration: row.duration,
            duration_seconds: row.duration_seconds,
            listen_url: row.listen_url,
            language: row.language,
            readers,
        }
    }
}
