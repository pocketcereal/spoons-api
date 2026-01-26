//! HTTP client factory for creating API clients with shared configuration.

use std::time::Duration;

use super::client::{ApiClient, AuthMethod};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub auth: AuthMethod,
}

impl ClientConfig {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            timeout: Duration::from_secs(30),
            auth: AuthMethod::None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> Result<ApiClient> {
        ApiClient::new(self.base_url, self.auth, self.timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_builder() {
        let config =
            ClientConfig::new("https://api.example.com").with_timeout(Duration::from_secs(60));

        assert_eq!(config.base_url, "https://api.example.com");
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(matches!(config.auth, AuthMethod::None));
    }
}
