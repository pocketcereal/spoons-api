//! Artist database model.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::db::schema::artists;
use crate::musicbrainz::{Area, Artist, LifeSpan};

/// Database row for artists table.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = artists)]
pub struct ArtistRow {
    pub id: Uuid,
    pub name: String,
    pub sort_name: Option<String>,
    pub artist_type: Option<String>,
    pub country: Option<String>,
    pub area_id: Option<Uuid>,
    pub disambiguation: Option<String>,
    pub life_span: Option<JsonValue>,
    pub cached_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insertable row for artists table.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = artists)]
pub struct NewArtistRow {
    pub id: Uuid,
    pub name: String,
    pub sort_name: Option<String>,
    pub artist_type: Option<String>,
    pub country: Option<String>,
    pub area_id: Option<Uuid>,
    pub disambiguation: Option<String>,
    pub life_span: Option<JsonValue>,
}

impl TryFrom<&Artist> for NewArtistRow {
    type Error = uuid::Error;

    fn try_from(artist: &Artist) -> Result<Self, Self::Error> {
        let area_id = artist
            .area
            .as_ref()
            .map(|a| Uuid::parse_str(&a.id))
            .transpose()?;

        let life_span = artist
            .life_span
            .as_ref()
            .and_then(|ls| serde_json::to_value(ls).ok());

        Ok(Self {
            id: Uuid::parse_str(&artist.id)?,
            name: artist.name.clone(),
            sort_name: artist.sort_name.clone(),
            artist_type: artist.artist_type.clone(),
            country: artist.country.clone(),
            area_id,
            disambiguation: artist.disambiguation.clone(),
            life_span,
        })
    }
}

impl ArtistRow {
    /// Convert to Artist domain type, optionally with area data.
    pub fn into_artist(self, area: Option<Area>) -> Artist {
        let life_span = self
            .life_span
            .and_then(|v| serde_json::from_value::<LifeSpan>(v).ok());

        Artist {
            id: self.id.to_string(),
            name: self.name,
            sort_name: self.sort_name,
            artist_type: self.artist_type,
            country: self.country,
            area,
            disambiguation: self.disambiguation,
            life_span,
        }
    }
}
