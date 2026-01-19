//! Configuration loading and management.

use serde::Deserialize;
use std::path::Path;

use crate::error::{AppError, Result};

/// Server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
}

const fn default_port() -> u16 {
    3000
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
        }
    }
}

/// Logging format options.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Text,
    Json,
}

/// Logging configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub format: LogFormat,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: LogFormat::default(),
        }
    }
}

/// MusicBrainz API configuration.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MusicBrainzConfig {
    #[serde(default = "default_musicbrainz_url")]
    pub url: String,
}

fn default_musicbrainz_url() -> String {
    "https://musicbrainz.org/ws/2/".to_string()
}

impl Default for MusicBrainzConfig {
    fn default() -> Self {
        Self {
            url: default_musicbrainz_url(),
        }
    }
}

/// Database configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL (can be overridden by DATABASE_URL env var).
    #[serde(default)]
    pub url: Option<String>,
    /// Maximum number of connections in the pool.
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    /// Cache TTL in seconds for MusicBrainz data.
    #[serde(default = "default_cache_ttl_seconds")]
    pub cache_ttl_seconds: i64,
}

const fn default_max_connections() -> usize {
    10
}

const fn default_cache_ttl_seconds() -> i64 {
    86400 // 24 hours
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: None,
            max_connections: default_max_connections(),
            cache_ttl_seconds: default_cache_ttl_seconds(),
        }
    }
}

/// Root application configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    #[allow(dead_code)] // Reserved for future use
    pub musicbrainz: MusicBrainzConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
}

/// Load configuration from a YAML file.
pub fn load_config(path: &Path) -> Result<AppConfig> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        AppError::Config(format!(
            "Failed to read config file '{}': {}",
            path.display(),
            e
        ))
    })?;

    serde_yaml::from_str(&content)
        .map_err(|e| AppError::Config(format!("Failed to parse config file: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.logging.format, LogFormat::Text);
    }

    #[test]
    fn test_parse_yaml_config() {
        let yaml = r#"
server:
  port: 8080
logging:
  level: debug
  format: json
"#;
        let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.logging.format, LogFormat::Json);
    }
}
