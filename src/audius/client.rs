//! Implements the Audius API with dynamic host resolution.
//! See: https://audiusproject.github.io/api-docs/

use rand::seq::SliceRandom;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::{AppError, Result};
use crate::http::{ClientConfig, DEFAULT_API_TIMEOUT, HOST_DISCOVERY_TIMEOUT};

use super::types::{AudiusResponse, AudiusTrack, AudiusUser, HostDiscoveryResponse};

const HOST_DISCOVERY_URL: &str = "https://api.audius.co";

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

#[derive(serde::Serialize)]
struct TrendingParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    app_name: Option<String>,
}

#[derive(serde::Serialize)]
struct AppNameParamOwned {
    #[serde(skip_serializing_if = "Option::is_none")]
    app_name: Option<String>,
}

/// Randomly selects from available hosts at startup and rotates on failure.
pub struct AudiusClient {
    app_name: String,
    hosts: Vec<String>,
    current_host_index: AtomicUsize,
    shared_client: reqwest::Client,
}

impl Clone for AudiusClient {
    fn clone(&self) -> Self {
        Self {
            app_name: self.app_name.clone(),
            hosts: self.hosts.clone(),
            current_host_index: AtomicUsize::new(self.current_host_index.load(Ordering::Relaxed)),
            shared_client: self.shared_client.clone(),
        }
    }
}

impl AudiusClient {
    /// Fetches available hosts from the discovery endpoint and randomly selects one.
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
            shared_client: Self::build_shared_client()?,
        })
    }

    pub fn with_host(host: &str, app_name: &str) -> Result<Self> {
        Ok(Self {
            app_name: app_name.to_string(),
            hosts: vec![host.to_string()],
            current_host_index: AtomicUsize::new(0),
            shared_client: Self::build_shared_client()?,
        })
    }

    fn build_shared_client() -> Result<reqwest::Client> {
        let user_agent = format!("spoons-api/{}", env!("CARGO_PKG_VERSION"));
        reqwest::Client::builder()
            .timeout(DEFAULT_API_TIMEOUT)
            .user_agent(&user_agent)
            .build()
            .map_err(|e| AppError::Internal(e.into()))
    }

    fn current_host(&self) -> String {
        let index = self.current_host_index.load(Ordering::Relaxed) % self.hosts.len();
        format!("{}/v1", &self.hosts[index])
    }

    fn rotate_host(&self) {
        let new_raw = self.current_host_index.fetch_add(1, Ordering::Relaxed) + 1;
        let new_index = new_raw % self.hosts.len();
        tracing::info!(host = %self.hosts[new_index], "Rotated to next Audius host after failure");
    }

    fn build_url(&self, path: &str) -> String {
        let base = self.current_host();
        let path = path.trim_start_matches('/');
        format!("{}/{}", base, path)
    }

    /// Falls back to other hosts on failure.
    async fn get_with_fallback<T: serde::de::DeserializeOwned, Q: serde::Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        let max_attempts = self.hosts.len();

        for attempt in 0..max_attempts {
            let url = self.build_url(path);
            let result = self.shared_client.get(&url).query(query).send().await;

            match result {
                Ok(response) if response.status().is_success() => {
                    return response
                        .json()
                        .await
                        .map_err(|e| AppError::Internal(e.into()));
                }
                Ok(response) => {
                    let status = response.status();
                    let is_last = attempt == max_attempts - 1;
                    if is_last || !status.is_server_error() {
                        let body = response.text().await.unwrap_or_default();
                        return Err(AppError::Server(format!(
                            "Audius request failed with status {}: {}",
                            status, body
                        )));
                    }
                    tracing::warn!(
                        status = %status,
                        attempt = attempt + 1,
                        max_attempts = max_attempts,
                        "Audius request failed, trying next host"
                    );
                    self.rotate_host();
                }
                Err(e) => {
                    let is_last = attempt == max_attempts - 1;
                    if is_last {
                        return Err(AppError::Internal(e.into()));
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

    async fn fetch_hosts() -> Result<Vec<String>> {
        let discovery_client = ClientConfig::new(HOST_DISCOVERY_URL)
            .with_timeout(HOST_DISCOVERY_TIMEOUT)
            .build()?;

        let response: HostDiscoveryResponse = discovery_client.get("").await?;

        tracing::debug!(count = response.data.len(), "Fetched Audius hosts");

        Ok(response.data)
    }

    pub fn hosts(&self) -> &[String] {
        &self.hosts
    }

    /// See: https://audiusproject.github.io/api-docs/#search-users
    pub async fn search_users(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<AudiusUser>> {
        let params = SearchParamsOwned {
            query: query.to_string(),
            limit: Some(limit),
            offset: Some(offset),
            app_name: Some(self.app_name.clone()),
        };
        let response: AudiusResponse<Vec<AudiusUser>> =
            self.get_with_fallback("/users/search", &params).await?;
        Ok(response.data)
    }

    /// See: https://audiusproject.github.io/api-docs/#get-user
    pub async fn get_user(&self, id: &str) -> Result<AudiusUser> {
        let path = format!("/users/{}", id);
        let params = AppNameParamOwned {
            app_name: Some(self.app_name.clone()),
        };
        let response: AudiusResponse<AudiusUser> = self.get_with_fallback(&path, &params).await?;
        Ok(response.data)
    }

    /// See: https://audiusproject.github.io/api-docs/#search-tracks
    pub async fn search_tracks(
        &self,
        query: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<AudiusTrack>> {
        let params = SearchParamsOwned {
            query: query.to_string(),
            limit: Some(limit),
            offset: Some(offset),
            app_name: Some(self.app_name.clone()),
        };
        let response: AudiusResponse<Vec<AudiusTrack>> =
            self.get_with_fallback("/tracks/search", &params).await?;
        Ok(response.data)
    }

    /// See: https://audiusproject.github.io/api-docs/#get-trending-tracks
    pub async fn trending_tracks(&self, limit: i32) -> Result<Vec<AudiusTrack>> {
        let params = TrendingParams {
            limit: Some(limit),
            app_name: Some(self.app_name.clone()),
        };
        let response: AudiusResponse<Vec<AudiusTrack>> =
            self.get_with_fallback("/tracks/trending", &params).await?;
        Ok(response.data)
    }

    /// See: https://audiusproject.github.io/api-docs/#get-track
    pub async fn get_track(&self, id: &str) -> Result<AudiusTrack> {
        let path = format!("/tracks/{}", id);
        let params = AppNameParamOwned {
            app_name: Some(self.app_name.clone()),
        };
        let response: AudiusResponse<AudiusTrack> = self.get_with_fallback(&path, &params).await?;
        Ok(response.data)
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

    #[test]
    fn test_trending_params_serialization() {
        let params = TrendingParams {
            limit: Some(10),
            app_name: Some("test-app".to_string()),
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["limit"], 10);
        assert_eq!(json["app_name"], "test-app");
    }

    #[test]
    fn test_trending_params_skips_none() {
        let params = TrendingParams {
            limit: None,
            app_name: None,
        };
        let json = serde_json::to_value(&params).unwrap();
        assert!(!json.as_object().unwrap().contains_key("limit"));
        assert!(!json.as_object().unwrap().contains_key("app_name"));
    }
}
