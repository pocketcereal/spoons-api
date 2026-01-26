//! Audius API client.
//!
//! Implements the Audius API with dynamic host resolution.
//! See: https://audiusproject.github.io/api-docs/

use rand::seq::SliceRandom;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::{AppError, Result};
use crate::http::{ApiClient, ClientConfig, DEFAULT_API_TIMEOUT, HOST_DISCOVERY_TIMEOUT};

use super::types::{AudiusResponse, AudiusTrack, AudiusUser, HostDiscoveryResponse};

/// Default Audius host discovery endpoint.
const HOST_DISCOVERY_URL: &str = "https://api.audius.co";

/// Search query parameters (owned version for async closures).
#[derive(serde::Serialize)]
struct SearchParamsOwned {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    app_name: Option<String>,
}

/// Simple app_name parameter (owned version for async closures).
#[derive(serde::Serialize)]
struct AppNameParamOwned {
    #[serde(skip_serializing_if = "Option::is_none")]
    app_name: Option<String>,
}

/// Audius API client with automatic host resolution and fallback.
///
/// Randomly selects from available hosts at startup and supports
/// rotation to other hosts on failure.
pub struct AudiusClient {
    /// Application name sent with requests.
    app_name: String,
    /// List of available hosts for requests.
    hosts: Vec<String>,
    /// Current host index for rotation on failure.
    current_host_index: AtomicUsize,
}

impl Clone for AudiusClient {
    fn clone(&self) -> Self {
        Self {
            app_name: self.app_name.clone(),
            hosts: self.hosts.clone(),
            current_host_index: AtomicUsize::new(self.current_host_index.load(Ordering::Relaxed)),
        }
    }
}

impl AudiusClient {
    /// Create a new Audius client.
    ///
    /// This will fetch available hosts from the Audius discovery endpoint
    /// and randomly select one for subsequent requests.
    pub async fn new(app_name: &str) -> Result<Self> {
        let mut hosts = Self::fetch_hosts().await?;

        if hosts.is_empty() {
            return Err(AppError::Server("No Audius hosts available".to_string()));
        }

        hosts.shuffle(&mut rand::thread_rng());

        let selected_host = &hosts[0];
        tracing::info!(host = %selected_host, total_hosts = hosts.len(), "Selected Audius API host");

        Ok(Self {
            app_name: app_name.to_string(),
            hosts,
            current_host_index: AtomicUsize::new(0),
        })
    }

    /// Create a client with a specific host (useful for testing).
    pub fn with_host(host: &str, app_name: &str) -> Result<Self> {
        Ok(Self {
            app_name: app_name.to_string(),
            hosts: vec![host.to_string()],
            current_host_index: AtomicUsize::new(0),
        })
    }

    /// Get the current host URL with API version path.
    fn current_host(&self) -> String {
        let index = self.current_host_index.load(Ordering::Relaxed);
        format!("{}/v1", &self.hosts[index])
    }

    /// Rotate to the next host, returning the new host URL.
    fn rotate_host(&self) -> String {
        let old_index = self.current_host_index.fetch_add(1, Ordering::Relaxed);
        let new_index = (old_index + 1) % self.hosts.len();
        self.current_host_index.store(new_index, Ordering::Relaxed);
        let host = format!("{}/v1", &self.hosts[new_index]);
        tracing::info!(host = %host, "Rotated to next Audius host after failure");
        host
    }

    /// Create an API client for the current host.
    fn create_client(&self) -> Result<ApiClient> {
        ClientConfig::new(self.current_host())
            .with_timeout(DEFAULT_API_TIMEOUT)
            .build()
    }

    /// Execute a request with fallback to other hosts on failure.
    async fn request_with_fallback<T, F, Fut>(&self, make_request: F) -> Result<T>
    where
        F: Fn(ApiClient) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let max_attempts = self.hosts.len();

        for attempt in 0..max_attempts {
            let client = self.create_client()?;
            match make_request(client).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let is_last = attempt == max_attempts - 1;
                    if is_last {
                        return Err(e);
                    }
                    tracing::warn!(
                        error = %e,
                        attempt = attempt + 1,
                        max_attempts = max_attempts,
                        "Audius request failed, trying next host"
                    );
                    self.rotate_host();
                }
            }
        }

        unreachable!("Loop should have returned")
    }

    /// Fetch available hosts from the Audius discovery endpoint.
    async fn fetch_hosts() -> Result<Vec<String>> {
        let discovery_client = ClientConfig::new(HOST_DISCOVERY_URL)
            .with_timeout(HOST_DISCOVERY_TIMEOUT)
            .build()?;

        let response: HostDiscoveryResponse = discovery_client.get("").await?;

        tracing::debug!(count = response.data.len(), "Fetched Audius hosts");

        Ok(response.data)
    }

    /// Get the list of available hosts (for debugging/introspection).
    pub fn hosts(&self) -> &[String] {
        &self.hosts
    }

    /// Search for users (artists).
    ///
    /// See: https://audiusproject.github.io/api-docs/#search-users
    pub async fn search_users(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<AudiusUser>> {
        let query = query.to_string();
        let app_name = self.app_name.clone();

        self.request_with_fallback(|client| {
            let query = query.clone();
            let app_name = app_name.clone();
            async move {
                let params = SearchParamsOwned {
                    query,
                    limit: Some(limit),
                    offset: Some(offset),
                    app_name: Some(app_name),
                };
                let response: AudiusResponse<Vec<AudiusUser>> =
                    client.get_with_query("/users/search", &params).await?;
                Ok(response.data)
            }
        })
        .await
    }

    /// Get a user by ID.
    ///
    /// See: https://audiusproject.github.io/api-docs/#get-user
    pub async fn get_user(&self, id: &str) -> Result<AudiusUser> {
        let path = format!("/users/{}", id);
        let app_name = self.app_name.clone();

        self.request_with_fallback(|client| {
            let path = path.clone();
            let app_name = app_name.clone();
            async move {
                let params = AppNameParamOwned {
                    app_name: Some(app_name),
                };
                let response: AudiusResponse<AudiusUser> =
                    client.get_with_query(&path, &params).await?;
                Ok(response.data)
            }
        })
        .await
    }

    /// Search for tracks.
    ///
    /// See: https://audiusproject.github.io/api-docs/#search-tracks
    pub async fn search_tracks(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<AudiusTrack>> {
        let query = query.to_string();
        let app_name = self.app_name.clone();

        self.request_with_fallback(|client| {
            let query = query.clone();
            let app_name = app_name.clone();
            async move {
                let params = SearchParamsOwned {
                    query,
                    limit: Some(limit),
                    offset: Some(offset),
                    app_name: Some(app_name),
                };
                let response: AudiusResponse<Vec<AudiusTrack>> =
                    client.get_with_query("/tracks/search", &params).await?;
                Ok(response.data)
            }
        })
        .await
    }

    /// Get a track by ID.
    ///
    /// See: https://audiusproject.github.io/api-docs/#get-track
    pub async fn get_track(&self, id: &str) -> Result<AudiusTrack> {
        let path = format!("/tracks/{}", id);
        let app_name = self.app_name.clone();

        self.request_with_fallback(|client| {
            let path = path.clone();
            let app_name = app_name.clone();
            async move {
                let params = AppNameParamOwned {
                    app_name: Some(app_name),
                };
                let response: AudiusResponse<AudiusTrack> =
                    client.get_with_query(&path, &params).await?;
                Ok(response.data)
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_with_host() {
        let client = AudiusClient::with_host("https://api.audius.co", "test-app");
        assert!(client.is_ok());
    }
}
