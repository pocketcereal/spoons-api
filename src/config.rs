use serde::Deserialize;
use std::path::Path;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 3000 }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Text,
    Json,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    /// Per-crate log level overrides (e.g. `["hyper=warn", "diesel=warn"]`).
    pub filters: Vec<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::default(),
            filters: vec![
                "tokio_postgres=warn".to_string(),
                "diesel=warn".to_string(),
                "diesel_async=warn".to_string(),
                "hyper=warn".to_string(),
                "hyper_util=warn".to_string(),
                "reqwest=warn".to_string(),
                "rustls=warn".to_string(),
                "h2=warn".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AudiusConfig {
    pub app_name: String,
    pub enabled: bool,
}

impl Default for AudiusConfig {
    fn default() -> Self {
        Self {
            app_name: "spoons-api".to_string(),
            enabled: true,
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    /// Can be overridden by DATABASE_URL env var.
    pub url: Option<String>,
    pub max_connections: usize,
    pub cache_ttl_seconds: i64,
}

impl std::fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("url", &self.url.as_ref().map(|_| "[REDACTED]"))
            .field("max_connections", &self.max_connections)
            .field("cache_ttl_seconds", &self.cache_ttl_seconds)
            .finish()
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: None,
            max_connections: 10,
            cache_ttl_seconds: 86400, // 24 hours
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct PodcastIndexConfig {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    /// Override for testing.
    pub base_url: String,
}

impl std::fmt::Debug for PodcastIndexConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodcastIndexConfig")
            .field("enabled", &self.enabled)
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .field("api_secret", &self.api_secret.as_ref().map(|_| "[REDACTED]"))
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl Default for PodcastIndexConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: None,
            api_secret: None,
            base_url: "https://api.podcastindex.org/api/1.0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LibriVoxConfig {
    pub enabled: bool,
    pub base_url: String,
}

impl Default for LibriVoxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "https://librivox.org/api/feed".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_entries: usize,
    pub trending_ttl_seconds: u64,
    pub search_ttl_seconds: u64,
    pub podcast_ttl_seconds: u64,
    pub episode_ttl_seconds: u64,
    pub categories_ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 1000,
            trending_ttl_seconds: 300,  // 5 minutes
            search_ttl_seconds: 600,    // 10 minutes
            podcast_ttl_seconds: 86400, // 24 hours
            episode_ttl_seconds: 3600,  // 1 hour
            categories_ttl_seconds: 86400, // 24 hours
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub audius: AudiusConfig,
    pub database: DatabaseConfig,
    pub podcast_index: PodcastIndexConfig,
    pub librivox: LibriVoxConfig,
    pub cache: CacheConfig,
}

/// Expands `${VAR}` references with environment variable values.
/// Unset variables are replaced with empty strings.
fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();
    let mut pos = 0;
    while let Some(start) = result[pos..].find("${") {
        let start = pos + start;
        let Some(end) = result[start..].find('}') else {
            break;
        };
        let end = start + end;
        let var_name = &result[start + 2..end];
        let value = std::env::var(var_name).unwrap_or_default();
        let value_len = value.len();
        result.replace_range(start..=end, &value);
        pos = start + value_len;
    }
    result
}

pub fn load_config(path: &Path) -> Result<AppConfig> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        AppError::Config(format!(
            "Failed to read config file '{}': {}",
            path.display(),
            e
        ))
    })?;

    let expanded = expand_env_vars(&content);

    serde_yaml::from_str(&expanded)
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
