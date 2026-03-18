use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudiobookSource {
    LibriVox,
}

impl AudiobookSource {
    pub fn as_prefix(&self) -> &'static str {
        match self {
            AudiobookSource::LibriVox => "librivox",
        }
    }

    pub fn format_id(&self, id: i64) -> String {
        format!("{}:{}", self.as_prefix(), id)
    }

    pub fn parse_id(prefixed_id: &str) -> Option<(AudiobookSource, i64)> {
        let parts: Vec<&str> = prefixed_id.splitn(2, ':').collect();
        if parts.len() != 2 {
            return None;
        }
        let source = AudiobookSource::from_str(parts[0]).ok()?;
        let id = parts[1].parse().ok()?;
        Some((source, id))
    }
}

impl fmt::Display for AudiobookSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_prefix())
    }
}

impl FromStr for AudiobookSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "librivox" => Ok(AudiobookSource::LibriVox),
            _ => Err(format!("Unknown audiobook source: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_id() {
        let source = AudiobookSource::LibriVox;
        assert_eq!(source.format_id(128), "librivox:128");
        assert_eq!(source.format_id(0), "librivox:0");
        assert_eq!(source.format_id(-1), "librivox:-1");
    }

    #[test]
    fn test_parse_id_valid() {
        let (source, id) = AudiobookSource::parse_id("librivox:128").unwrap();
        assert_eq!(source, AudiobookSource::LibriVox);
        assert_eq!(id, 128);

        let (source, id) = AudiobookSource::parse_id("librivox:0").unwrap();
        assert_eq!(source, AudiobookSource::LibriVox);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_parse_id_invalid() {
        assert!(AudiobookSource::parse_id("invalid").is_none());
        assert!(AudiobookSource::parse_id("librivox").is_none());
        assert!(AudiobookSource::parse_id("librivox:").is_none());
        assert!(AudiobookSource::parse_id("librivox:abc").is_none());
        assert!(AudiobookSource::parse_id("unknown:128").is_none());
        assert!(AudiobookSource::parse_id(":128").is_none());
        assert!(AudiobookSource::parse_id("").is_none());
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            AudiobookSource::from_str("librivox").unwrap(),
            AudiobookSource::LibriVox
        );
        assert_eq!(
            AudiobookSource::from_str("LIBRIVOX").unwrap(),
            AudiobookSource::LibriVox
        );
        assert_eq!(
            AudiobookSource::from_str("LibriVox").unwrap(),
            AudiobookSource::LibriVox
        );

        assert!(AudiobookSource::from_str("unknown").is_err());
        assert!(AudiobookSource::from_str("spotify").is_err());
        assert!(AudiobookSource::from_str("").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(AudiobookSource::LibriVox.to_string(), "librivox");
    }

    #[test]
    fn test_round_trip() {
        let source = AudiobookSource::LibriVox;
        let id = 99999;

        let formatted = source.format_id(id);
        let (parsed_source, parsed_id) = AudiobookSource::parse_id(&formatted).unwrap();

        assert_eq!(parsed_source, source);
        assert_eq!(parsed_id, id);
    }
}
