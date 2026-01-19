//! Release database model.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::schema::releases;
use crate::musicbrainz::{Release, ReleaseGroup};

/// Database row for releases table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = releases)]
pub struct ReleaseRow {
    pub id: Uuid,
    pub title: String,
    pub status: Option<String>,
    pub release_date: Option<String>,
    pub country: Option<String>,
    pub barcode: Option<String>,
    pub disambiguation: Option<String>,
    pub release_group_id: Option<Uuid>,
    pub cached_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insertable row for releases table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = releases)]
pub struct NewReleaseRow {
    pub id: Uuid,
    pub title: String,
    pub status: Option<String>,
    pub release_date: Option<String>,
    pub country: Option<String>,
    pub barcode: Option<String>,
    pub disambiguation: Option<String>,
    pub release_group_id: Option<Uuid>,
}

impl TryFrom<&Release> for NewReleaseRow {
    type Error = uuid::Error;

    fn try_from(release: &Release) -> Result<Self, Self::Error> {
        let release_group_id = release
            .release_group
            .as_ref()
            .map(|rg| Uuid::parse_str(&rg.id))
            .transpose()?;

        Ok(Self {
            id: Uuid::parse_str(&release.id)?,
            title: release.title.clone(),
            status: release.status.clone(),
            release_date: release.date.clone(),
            country: release.country.clone(),
            barcode: release.barcode.clone(),
            disambiguation: release.disambiguation.clone(),
            release_group_id,
        })
    }
}

impl ReleaseRow {
    /// Convert to Release domain type, optionally with release group data.
    pub fn into_release(self, release_group: Option<ReleaseGroup>) -> Release {
        Release {
            id: self.id.to_string(),
            title: self.title,
            status: self.status,
            date: self.release_date,
            country: self.country,
            barcode: self.barcode,
            disambiguation: self.disambiguation,
            release_group,
        }
    }
}
