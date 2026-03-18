//! MusicBrainz API response types.

use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

/// Artist entity from MusicBrainz.
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Artist {
    /// MusicBrainz ID.
    pub id: String,
    /// Artist name.
    pub name: String,
    /// Sort name for the artist.
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
    /// Artist type (person, group, etc.).
    #[serde(rename = "type")]
    pub artist_type: Option<String>,
    /// Country of origin.
    pub country: Option<String>,
    /// Area of origin.
    pub area: Option<Area>,
    /// Disambiguation comment.
    pub disambiguation: Option<String>,
    /// Life span information.
    #[serde(rename = "life-span")]
    pub life_span: Option<LifeSpan>,
}

/// Area entity from MusicBrainz.
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Area {
    /// MusicBrainz ID.
    pub id: String,
    /// Area name.
    pub name: String,
    /// Sort name for the area.
    #[serde(rename = "sort-name")]
    pub sort_name: Option<String>,
}

/// Life span information for an artist.
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct LifeSpan {
    /// Begin date.
    pub begin: Option<String>,
    /// End date.
    pub end: Option<String>,
    /// Whether the entity has ended.
    pub ended: Option<bool>,
}

/// Release entity from MusicBrainz.
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Release {
    /// MusicBrainz ID.
    pub id: String,
    /// Release title.
    pub title: String,
    /// Release status.
    pub status: Option<String>,
    /// Release date.
    pub date: Option<String>,
    /// Country of release.
    pub country: Option<String>,
    /// Barcode.
    pub barcode: Option<String>,
    /// Disambiguation comment.
    pub disambiguation: Option<String>,
    /// Release group.
    #[serde(rename = "release-group")]
    pub release_group: Option<ReleaseGroup>,
}

/// Release group entity from MusicBrainz.
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ReleaseGroup {
    /// MusicBrainz ID.
    pub id: String,
    /// Release group title.
    pub title: String,
    /// Primary type (album, single, EP, etc.).
    #[serde(rename = "primary-type")]
    pub primary_type: Option<String>,
    /// Secondary types.
    #[serde(rename = "secondary-types")]
    pub secondary_types: Option<Vec<String>>,
    /// First release date.
    #[serde(rename = "first-release-date")]
    pub first_release_date: Option<String>,
    /// Disambiguation comment.
    pub disambiguation: Option<String>,
}

/// Recording entity from MusicBrainz.
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Recording {
    /// MusicBrainz ID.
    pub id: String,
    /// Recording title.
    pub title: String,
    /// Recording length in milliseconds.
    pub length: Option<i64>,
    /// Disambiguation comment.
    pub disambiguation: Option<String>,
    /// Video flag.
    pub video: Option<bool>,
    /// Artist credits.
    #[serde(rename = "artist-credit", default)]
    pub artist_credit: Vec<ArtistCredit>,
}

/// Artist credit entry from MusicBrainz.
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ArtistCredit {
    /// The credited artist.
    pub artist: ArtistCreditArtist,
    /// Join phrase (e.g., " & ", " feat. ").
    #[serde(default)]
    pub joinphrase: String,
}

/// Minimal artist info within an artist credit.
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ArtistCreditArtist {
    /// MusicBrainz ID.
    pub id: String,
    /// Artist name.
    pub name: String,
}

/// Search result wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T> {
    /// Creation timestamp.
    pub created: Option<String>,
    /// Total number of results.
    pub count: i64,
    /// Offset of current results.
    pub offset: i64,
    /// Results list (varies by entity type).
    #[serde(flatten)]
    pub data: SearchData<T>,
}

/// Search data container (handles different entity type keys).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SearchData<T> {
    /// Artists search result.
    Artists { artists: Vec<T> },
    /// Releases search result.
    Releases { releases: Vec<T> },
    /// Recordings search result.
    Recordings { recordings: Vec<T> },
    /// Release groups search result.
    #[serde(rename = "release-groups")]
    ReleaseGroups {
        #[serde(rename = "release-groups")]
        release_groups: Vec<T>,
    },
}

impl<T> SearchResult<T> {
    /// Get the items from the search result.
    pub fn items(self) -> Vec<T> {
        match self.data {
            SearchData::Artists { artists } => artists,
            SearchData::Releases { releases } => releases,
            SearchData::Recordings { recordings } => recordings,
            SearchData::ReleaseGroups { release_groups } => release_groups,
        }
    }

    /// Get the total count and items from the search result.
    pub fn into_parts(self) -> (i64, Vec<T>) {
        (self.count, self.items())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artist_deserialize() {
        let json = r#"{
            "id": "5b11f4ce-a62d-471e-81fc-a69a8278c7da",
            "name": "Nirvana",
            "sort-name": "Nirvana",
            "type": "Group",
            "country": "US"
        }"#;
        let artist: Artist = serde_json::from_str(json).unwrap();
        assert_eq!(artist.name, "Nirvana");
        assert_eq!(artist.artist_type, Some("Group".to_string()));
    }

    #[test]
    fn test_recording_deserialize() {
        let json = r#"{
            "id": "abc123",
            "title": "Smells Like Teen Spirit",
            "length": 301000
        }"#;
        let recording: Recording = serde_json::from_str(json).unwrap();
        assert_eq!(recording.title, "Smells Like Teen Spirit");
        assert_eq!(recording.length, Some(301000));
    }

    #[test]
    fn test_into_parts_preserves_count_and_items() {
        let json = r#"{
            "created": "2024-01-01",
            "count": 42,
            "offset": 0,
            "artists": [
                {"id": "1", "name": "Artist One", "sort-name": "One, Artist"},
                {"id": "2", "name": "Artist Two", "sort-name": "Two, Artist"}
            ]
        }"#;
        let result: SearchResult<Artist> = serde_json::from_str(json).unwrap();
        let (count, items) = result.into_parts();
        assert_eq!(count, 42);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "Artist One");
    }

    #[test]
    fn test_into_parts_empty_results() {
        let json = r#"{
            "count": 0,
            "offset": 0,
            "recordings": []
        }"#;
        let result: SearchResult<Recording> = serde_json::from_str(json).unwrap();
        let (count, items) = result.into_parts();
        assert_eq!(count, 0);
        assert!(items.is_empty());
    }
}
