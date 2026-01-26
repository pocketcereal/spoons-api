//! Data source identification.

use async_graphql::Enum;
use serde::{Deserialize, Serialize};

/// Identifies the source of music data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Enum)]
pub enum DataSource {
    /// MusicBrainz open music encyclopedia.
    MusicBrainz,
    /// Audius decentralized music streaming platform.
    Audius,
}

impl std::fmt::Display for DataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSource::MusicBrainz => write!(f, "musicbrainz"),
            DataSource::Audius => write!(f, "audius"),
        }
    }
}

impl DataSource {
    /// Returns the source identifier as a lowercase string.
    pub fn as_str(&self) -> &'static str {
        match self {
            DataSource::MusicBrainz => "musicbrainz",
            DataSource::Audius => "audius",
        }
    }

    /// Creates a prefixed ID in the format "source:id".
    ///
    /// This provides a consistent ID format across all data sources,
    /// allowing disambiguation when combining results from multiple sources.
    pub fn prefix_id(&self, id: &str) -> String {
        format!("{}:{}", self.as_str(), id)
    }
}
