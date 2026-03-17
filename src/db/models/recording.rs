use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::schema::recordings;
use crate::musicbrainz::{ArtistCredit, Recording};

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = recordings)]
pub struct RecordingRow {
    pub id: Uuid,
    pub title: String,
    pub length_ms: Option<i64>,
    pub disambiguation: Option<String>,
    pub video: Option<bool>,
    pub artist_credit: serde_json::Value,
    pub cached_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = recordings)]
pub struct NewRecordingRow {
    pub id: Uuid,
    pub title: String,
    pub length_ms: Option<i64>,
    pub disambiguation: Option<String>,
    pub video: Option<bool>,
    pub artist_credit: serde_json::Value,
}

#[derive(Debug)]
pub enum RecordingConversionError {
    InvalidUuid(uuid::Error),
    SerializationFailed(serde_json::Error),
}

impl std::fmt::Display for RecordingConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUuid(e) => write!(f, "Invalid UUID: {}", e),
            Self::SerializationFailed(e) => write!(f, "Serialization failed: {}", e),
        }
    }
}

impl From<uuid::Error> for RecordingConversionError {
    fn from(e: uuid::Error) -> Self {
        Self::InvalidUuid(e)
    }
}

impl TryFrom<&Recording> for NewRecordingRow {
    type Error = RecordingConversionError;

    fn try_from(recording: &Recording) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::parse_str(&recording.id)?,
            title: recording.title.clone(),
            length_ms: recording.length,
            disambiguation: recording.disambiguation.clone(),
            video: recording.video,
            artist_credit: serde_json::to_value(&recording.artist_credit)
                .map_err(RecordingConversionError::SerializationFailed)?,
        })
    }
}

impl From<RecordingRow> for Recording {
    fn from(row: RecordingRow) -> Self {
        let artist_credit: Vec<ArtistCredit> =
            serde_json::from_value(row.artist_credit).unwrap_or_else(|e| {
                tracing::warn!(recording_id = %row.id, error = %e, "Failed to deserialize artist_credit, defaulting to empty");
                Vec::new()
            });
        Self {
            id: row.id.to_string(),
            title: row.title,
            length: row.length_ms,
            disambiguation: row.disambiguation,
            video: row.video,
            artist_credit,
        }
    }
}
