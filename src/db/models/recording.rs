//! Recording database model.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::schema::recordings;
use crate::musicbrainz::Recording;

/// Database row for recordings table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = recordings)]
pub struct RecordingRow {
    pub id: Uuid,
    pub title: String,
    pub length_ms: Option<i64>,
    pub disambiguation: Option<String>,
    pub video: Option<bool>,
    pub cached_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insertable row for recordings table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = recordings)]
pub struct NewRecordingRow {
    pub id: Uuid,
    pub title: String,
    pub length_ms: Option<i64>,
    pub disambiguation: Option<String>,
    pub video: Option<bool>,
}

impl TryFrom<&Recording> for NewRecordingRow {
    type Error = uuid::Error;

    fn try_from(recording: &Recording) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::parse_str(&recording.id)?,
            title: recording.title.clone(),
            length_ms: recording.length,
            disambiguation: recording.disambiguation.clone(),
            video: recording.video,
        })
    }
}

impl From<RecordingRow> for Recording {
    fn from(row: RecordingRow) -> Self {
        Self {
            id: row.id.to_string(),
            title: row.title,
            length: row.length_ms,
            disambiguation: row.disambiguation,
            video: row.video,
        }
    }
}
