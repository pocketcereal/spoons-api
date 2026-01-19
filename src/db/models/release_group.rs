//! Release group database model.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::db::schema::release_groups;
use crate::musicbrainz::ReleaseGroup;

/// Database row for release_groups table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = release_groups)]
pub struct ReleaseGroupRow {
    pub id: Uuid,
    pub title: String,
    pub primary_type: Option<String>,
    pub secondary_types: Option<JsonValue>,
    pub first_release_date: Option<String>,
    pub disambiguation: Option<String>,
    pub cached_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insertable row for release_groups table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = release_groups)]
pub struct NewReleaseGroupRow {
    pub id: Uuid,
    pub title: String,
    pub primary_type: Option<String>,
    pub secondary_types: Option<JsonValue>,
    pub first_release_date: Option<String>,
    pub disambiguation: Option<String>,
}

impl TryFrom<&ReleaseGroup> for NewReleaseGroupRow {
    type Error = uuid::Error;

    fn try_from(rg: &ReleaseGroup) -> Result<Self, Self::Error> {
        let secondary_types = rg
            .secondary_types
            .as_ref()
            .and_then(|st| serde_json::to_value(st).ok());

        Ok(Self {
            id: Uuid::parse_str(&rg.id)?,
            title: rg.title.clone(),
            primary_type: rg.primary_type.clone(),
            secondary_types,
            first_release_date: rg.first_release_date.clone(),
            disambiguation: rg.disambiguation.clone(),
        })
    }
}

impl From<ReleaseGroupRow> for ReleaseGroup {
    fn from(row: ReleaseGroupRow) -> Self {
        let secondary_types = row
            .secondary_types
            .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok());

        Self {
            id: row.id.to_string(),
            title: row.title,
            primary_type: row.primary_type,
            secondary_types,
            first_release_date: row.first_release_date,
            disambiguation: row.disambiguation,
        }
    }
}
