//! Area database model.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::schema::areas;
use crate::musicbrainz::Area;

/// Database row for areas table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = areas)]
pub struct AreaRow {
    pub id: Uuid,
    pub name: String,
    pub sort_name: Option<String>,
    pub cached_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insertable row for areas table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = areas)]
pub struct NewAreaRow {
    pub id: Uuid,
    pub name: String,
    pub sort_name: Option<String>,
}

impl TryFrom<&Area> for NewAreaRow {
    type Error = uuid::Error;

    fn try_from(area: &Area) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::parse_str(&area.id)?,
            name: area.name.clone(),
            sort_name: area.sort_name.clone(),
        })
    }
}

impl From<AreaRow> for Area {
    fn from(row: AreaRow) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            sort_name: row.sort_name,
        }
    }
}
