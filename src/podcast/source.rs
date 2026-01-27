use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Sources for podcast data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PodcastSource {
    PodcastIndex,
    // Future: Spotify, Apple, etc.
}

impl PodcastSource {
    /// Get the string prefix used in IDs
    pub fn as_prefix(&self) -> &'static str {
        match self {
            PodcastSource::PodcastIndex => "podcastindex",
        }
    }

    /// Format an ID with source prefix
    pub fn format_id(&self, id: i64) -> String {
        format!("{}:{}", self.as_prefix(), id)
    }

    /// Parse a prefixed ID, returning (source, id)
    pub fn parse_id(prefixed_id: &str) -> Option<(PodcastSource, i64)> {
        let parts: Vec<&str> = prefixed_id.splitn(2, ':').collect();
        if parts.len() != 2 {
            return None;
        }
        let source = PodcastSource::from_str(parts[0]).ok()?;
        let id = parts[1].parse().ok()?;
        Some((source, id))
    }
}

impl fmt::Display for PodcastSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_prefix())
    }
}

impl FromStr for PodcastSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "podcastindex" => Ok(PodcastSource::PodcastIndex),
            _ => Err(format!("Unknown podcast source: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_id() {
        let source = PodcastSource::PodcastIndex;
        assert_eq!(source.format_id(12345), "podcastindex:12345");
        assert_eq!(source.format_id(0), "podcastindex:0");
        assert_eq!(source.format_id(-1), "podcastindex:-1");
    }

    #[test]
    fn test_parse_id_valid() {
        let (source, id) = PodcastSource::parse_id("podcastindex:12345").unwrap();
        assert_eq!(source, PodcastSource::PodcastIndex);
        assert_eq!(id, 12345);

        let (source, id) = PodcastSource::parse_id("podcastindex:0").unwrap();
        assert_eq!(source, PodcastSource::PodcastIndex);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_parse_id_invalid() {
        assert!(PodcastSource::parse_id("invalid").is_none());
        assert!(PodcastSource::parse_id("podcastindex").is_none());
        assert!(PodcastSource::parse_id("podcastindex:").is_none());
        assert!(PodcastSource::parse_id("podcastindex:abc").is_none());
        assert!(PodcastSource::parse_id("unknown:12345").is_none());
        assert!(PodcastSource::parse_id(":12345").is_none());
        assert!(PodcastSource::parse_id("").is_none());
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            PodcastSource::from_str("podcastindex").unwrap(),
            PodcastSource::PodcastIndex
        );
        assert_eq!(
            PodcastSource::from_str("PODCASTINDEX").unwrap(),
            PodcastSource::PodcastIndex
        );
        assert_eq!(
            PodcastSource::from_str("PodcastIndex").unwrap(),
            PodcastSource::PodcastIndex
        );

        assert!(PodcastSource::from_str("unknown").is_err());
        assert!(PodcastSource::from_str("spotify").is_err());
        assert!(PodcastSource::from_str("").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(PodcastSource::PodcastIndex.to_string(), "podcastindex");
    }

    #[test]
    fn test_round_trip() {
        let source = PodcastSource::PodcastIndex;
        let id = 99999;

        let formatted = source.format_id(id);
        let (parsed_source, parsed_id) = PodcastSource::parse_id(&formatted).unwrap();

        assert_eq!(parsed_source, source);
        assert_eq!(parsed_id, id);
    }
}
