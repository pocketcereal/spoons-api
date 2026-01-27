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

/// Audius API configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct AudiusConfig {
    /// Application name sent with requests.
    #[serde(default = "default_audius_app_name")]
    pub app_name: String,
    /// Whether to enable Audius integration.
    #[serde(default = "default_audius_enabled")]
    pub enabled: bool,
}

fn default_audius_app_name() -> String {
    "spoons-api".to_string()
}

const fn default_audius_enabled() -> bool {
    true
}

impl Default for AudiusConfig {
    fn default() -> Self {
        Self {
            app_name: default_audius_app_name(),
            enabled: default_audius_enabled(),
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

/// PodcastIndex API configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct PodcastIndexConfig {
    /// Whether PodcastIndex integration is enabled
    #[serde(default)]
    pub enabled: bool,
    /// PodcastIndex API key
    pub api_key: Option<String>,
    /// PodcastIndex API secret
    pub api_secret: Option<String>,
    /// Base URL override (for testing)
    #[serde(default = "default_podcast_index_url")]
    pub base_url: String,
}

fn default_podcast_index_url() -> String {
    "https://api.podcastindex.org/api/1.0".to_string()
}

impl Default for PodcastIndexConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: None,
            api_secret: None,
            base_url: default_podcast_index_url(),
        }
    }
}

/// Cache configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    /// Whether in-memory caching is enabled
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    /// Maximum number of entries in the cache
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
    /// TTL for trending podcasts (seconds)
    #[serde(default = "default_trending_ttl")]
    pub trending_ttl_seconds: u64,
    /// TTL for search results (seconds)
    #[serde(default = "default_search_ttl")]
    pub search_ttl_seconds: u64,
    /// TTL for podcast details (seconds)
    #[serde(default = "default_podcast_ttl")]
    pub podcast_ttl_seconds: u64,
    /// TTL for episode data (seconds)
    #[serde(default = "default_episode_ttl")]
    pub episode_ttl_seconds: u64,
    /// TTL for categories (seconds)
    #[serde(default = "default_categories_ttl")]
    pub categories_ttl_seconds: u64,
}

fn default_cache_enabled() -> bool {
    true
}

const fn default_max_entries() -> usize {
    1000
}

const fn default_trending_ttl() -> u64 {
    300 // 5 minutes
}

const fn default_search_ttl() -> u64 {
    600 // 10 minutes
}

const fn default_podcast_ttl() -> u64 {
    86400 // 24 hours
}

const fn default_episode_ttl() -> u64 {
    3600 // 1 hour
}

const fn default_categories_ttl() -> u64 {
    86400 // 24 hours
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            max_entries: default_max_entries(),
            trending_ttl_seconds: default_trending_ttl(),
            search_ttl_seconds: default_search_ttl(),
            podcast_ttl_seconds: default_podcast_ttl(),
            episode_ttl_seconds: default_episode_ttl(),
            categories_ttl_seconds: default_categories_ttl(),
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
    pub audius: AudiusConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub podcast_index: PodcastIndexConfig,
    #[serde(default)]
    pub cache: CacheConfig,
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

    #[test]
    fn test_podcast_index_config_defaults() {
        let config = PodcastIndexConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.api_key, None);
        assert_eq!(config.api_secret, None);
        assert_eq!(config.base_url, "https://api.podcastindex.org/api/1.0");
    }

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_entries, 1000);
        assert_eq!(config.trending_ttl_seconds, 300);
        assert_eq!(config.search_ttl_seconds, 600);
        assert_eq!(config.podcast_ttl_seconds, 86400);
        assert_eq!(config.episode_ttl_seconds, 3600);
        assert_eq!(config.categories_ttl_seconds, 86400);
    }

    #[test]
    fn test_parse_podcast_index_config() {
        let yaml = r#"
podcast_index:
  enabled: true
  api_key: test_key
  api_secret: test_secret
  base_url: https://test.example.com
"#;
        let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.podcast_index.enabled);
        assert_eq!(config.podcast_index.api_key, Some("test_key".to_string()));
        assert_eq!(
            config.podcast_index.api_secret,
            Some("test_secret".to_string())
        );
        assert_eq!(config.podcast_index.base_url, "https://test.example.com");
    }

    #[test]
    fn test_parse_cache_config() {
        let yaml = r#"
cache:
  enabled: false
  max_entries: 500
  trending_ttl_seconds: 150
  search_ttl_seconds: 300
  podcast_ttl_seconds: 43200
  episode_ttl_seconds: 1800
  categories_ttl_seconds: 43200
"#;
        let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(!config.cache.enabled);
        assert_eq!(config.cache.max_entries, 500);
        assert_eq!(config.cache.trending_ttl_seconds, 150);
        assert_eq!(config.cache.search_ttl_seconds, 300);
        assert_eq!(config.cache.podcast_ttl_seconds, 43200);
        assert_eq!(config.cache.episode_ttl_seconds, 1800);
        assert_eq!(config.cache.categories_ttl_seconds, 43200);
    }
}
