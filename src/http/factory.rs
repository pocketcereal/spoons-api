//! HTTP client factory for creating API clients with shared configuration.

#![allow(dead_code)]

use std::time::Duration;

use super::client::{ApiClient, AuthMethod};
use crate::error::Result;

/// Configuration for creating API clients.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL for the API.
    pub base_url: String,
    /// Request timeout duration.
    pub timeout: Duration,
    /// Authentication method.
    pub auth: AuthMethod,
}

impl ClientConfig {
    /// Create a new client config with default timeout.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            timeout: Duration::from_secs(30),
            auth: AuthMethod::None,
        }
    }

    /// Set the timeout duration.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set bearer token authentication.
    pub fn with_bearer_token(mut self, token: impl Into<String>) -> Self {
        self.auth = AuthMethod::Bearer(token.into());
        self
    }

    /// Set API key authentication.
    pub fn with_api_key(mut self, header: impl Into<String>, key: impl Into<String>) -> Self {
        self.auth = AuthMethod::ApiKey {
            header: header.into(),
            key: key.into(),
        };
        self
    }

    /// Set basic authentication.
    pub fn with_basic_auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.auth = AuthMethod::Basic {
            username: username.into(),
            password: password.into(),
        };
        self
    }

    /// Build an API client from this config.
    pub fn build(self) -> Result<ApiClient> {
        ApiClient::new(self.base_url, self.auth, self.timeout)
    }
}

/// Factory for creating multiple API clients with shared base configuration.
#[derive(Debug, Clone)]
pub struct ClientFactory {
    /// Default timeout for all clients.
    default_timeout: Duration,
}

impl Default for ClientFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientFactory {
    /// Create a new client factory.
    pub fn new() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
        }
    }

    /// Set the default timeout for all created clients.
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Create a client config with the factory's defaults.
    pub fn config(&self, base_url: impl Into<String>) -> ClientConfig {
        ClientConfig::new(base_url).with_timeout(self.default_timeout)
    }

    /// Create an unauthenticated client.
    pub fn create(&self, base_url: impl Into<String>) -> Result<ApiClient> {
        self.config(base_url).build()
    }

    /// Create a client with bearer token authentication.
    pub fn create_with_bearer(
        &self,
        base_url: impl Into<String>,
        token: impl Into<String>,
    ) -> Result<ApiClient> {
        self.config(base_url).with_bearer_token(token).build()
    }

    /// Create a client with API key authentication.
    pub fn create_with_api_key(
        &self,
        base_url: impl Into<String>,
        header: impl Into<String>,
        key: impl Into<String>,
    ) -> Result<ApiClient> {
        self.config(base_url).with_api_key(header, key).build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_builder() {
        let config = ClientConfig::new("https://api.example.com")
            .with_timeout(Duration::from_secs(60))
            .with_bearer_token("test-token");

        assert_eq!(config.base_url, "https://api.example.com");
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(matches!(config.auth, AuthMethod::Bearer(_)));
    }

    #[test]
    fn test_client_factory() {
        let factory = ClientFactory::new().with_default_timeout(Duration::from_secs(45));

        let client = factory.create("https://api.example.com");
        assert!(client.is_ok());
    }
}
