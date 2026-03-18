use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::Value as JsonValue;

use crate::audiobook::{Audiobook, AudiobookAuthor};
use crate::db::schema::audiobooks;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = audiobooks)]
pub struct AudiobookRow {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub copyright_year: Option<String>,
    pub num_sections: Option<i32>,
    pub total_time: Option<String>,
    pub total_time_secs: Option<i64>,
    pub authors: JsonValue,
    pub url_text_source: Option<String>,
    pub url_zip_file: Option<String>,
    pub url_librivox: Option<String>,
    pub url_iarchive: Option<String>,
    pub coverart_url: Option<String>,
    pub coverart_thumbnail: Option<String>,
    pub cached_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = audiobooks)]
pub struct NewAudiobookRow {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub copyright_year: Option<String>,
    pub num_sections: Option<i32>,
    pub total_time: Option<String>,
    pub total_time_secs: Option<i64>,
    pub authors: JsonValue,
    pub url_text_source: Option<String>,
    pub url_zip_file: Option<String>,
    pub url_librivox: Option<String>,
    pub url_iarchive: Option<String>,
    pub coverart_url: Option<String>,
    pub coverart_thumbnail: Option<String>,
}

impl From<&Audiobook> for NewAudiobookRow {
    fn from(audiobook: &Audiobook) -> Self {
        let authors =
            serde_json::to_value(&audiobook.authors).unwrap_or_else(|_| JsonValue::Array(vec![]));

        Self {
            id: audiobook.id,
            title: audiobook.title.clone(),
            description: audiobook.description.clone(),
            language: audiobook.language.clone(),
            copyright_year: audiobook.copyright_year.clone(),
            num_sections: audiobook.num_sections,
            total_time: audiobook.total_time.clone(),
            total_time_secs: audiobook.total_time_secs,
            authors,
            url_text_source: audiobook.url_text_source.clone(),
            url_zip_file: audiobook.url_zip_file.clone(),
            url_librivox: audiobook.url_librivox.clone(),
            url_iarchive: audiobook.url_iarchive.clone(),
            coverart_url: audiobook.coverart_url.clone(),
            coverart_thumbnail: audiobook.coverart_thumbnail.clone(),
        }
    }
}

impl From<AudiobookRow> for Audiobook {
    fn from(row: AudiobookRow) -> Self {
        let authors =
            serde_json::from_value::<Vec<AudiobookAuthor>>(row.authors).unwrap_or_default();

        Self {
            id: row.id,
            title: row.title,
            description: row.description,
            language: row.language,
            copyright_year: row.copyright_year,
            num_sections: row.num_sections,
            total_time: row.total_time,
            total_time_secs: row.total_time_secs,
            authors,
            url_text_source: row.url_text_source,
            url_zip_file: row.url_zip_file,
            url_librivox: row.url_librivox,
            url_iarchive: row.url_iarchive,
            coverart_url: row.coverart_url,
            coverart_thumbnail: row.coverart_thumbnail,
        }
    }
}
