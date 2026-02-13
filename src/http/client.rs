//! API client wrapper with connection pooling and retry logic.
//!
//! The underlying `reqwest::Client` maintains an internal connection pool
//! that reuses connections across requests. This struct should be cloned
//! and shared across the application rather than recreated per request.

use reqwest::header::HeaderMap;
use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::time::{Duration, Instant};

use crate::error::{AppError, Result};

/// Default retry configuration: 3 retries with exponential backoff.
const DEFAULT_MAX_RETRIES: u32 = 3;
/// Initial retry delay of 100ms.
const DEFAULT_INITIAL_RETRY_DELAY_MS: u64 = 100;

/// Retry configuration for HTTP requests.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (not including initial request).
    pub max_retries: u32,
    /// Initial delay before first retry (doubles on each subsequent retry).
    pub initial_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            initial_delay: Duration::from_millis(DEFAULT_INITIAL_RETRY_DELAY_MS),
        }
    }
}

/// API client wrapper with connection pooling and automatic retry.
///
/// The underlying `reqwest::Client` maintains an internal connection pool
/// that reuses connections across requests. This struct should be cloned
/// and shared across the application rather than recreated per request.
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    retry_config: RetryConfig,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>, timeout: Duration) -> Result<Self> {
        let user_agent = format!("spoons-api/{}", env!("CARGO_PKG_VERSION"));

        let client = Client::builder()
            .timeout(timeout)
            .user_agent(&user_agent)
            .build()
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(Self {
            client,
            base_url: base_url.into(),
            retry_config: RetryConfig::default(),
        })
    }

    fn build_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{}/{}", base, path)
    }

    /// Determine if an error is retryable (timeouts and connection errors only).
    /// Note: is_request() is intentionally excluded as it includes client errors (4xx)
    /// that should not be retried.
    fn is_retryable_error(err: &reqwest::Error) -> bool {
        err.is_timeout() || err.is_connect()
    }

    /// Determine if a status code is retryable (5xx server errors).
    fn is_retryable_status(status: StatusCode) -> bool {
        status.is_server_error()
    }

    /// Calculate delay for a retry attempt using exponential backoff.
    fn retry_delay(&self, attempt: u32) -> Duration {
        self.retry_config.initial_delay * 2u32.saturating_pow(attempt)
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.get_internal(path, None::<&()>, None).await
    }

    pub async fn get_with_query<T: DeserializeOwned, Q: serde::Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        self.get_internal(path, Some(query), None).await
    }

    /// Makes a GET request with query parameters and extra headers.
    ///
    /// Use this when you need to inject per-request headers (e.g., for authentication).
    pub async fn get_with_headers<T: DeserializeOwned, Q: serde::Serialize>(
        &self,
        path: &str,
        query: &Q,
        headers: HeaderMap,
    ) -> Result<T> {
        self.get_internal(path, Some(query), Some(headers)).await
    }

    async fn get_internal<T: DeserializeOwned, Q: serde::Serialize>(
        &self,
        path: &str,
        query: Option<&Q>,
        extra_headers: Option<HeaderMap>,
    ) -> Result<T> {
        let url = self.build_url(path);
        let max_attempts = self.retry_config.max_retries + 1;
        let mut last_error: Option<AppError> = None;

        for attempt in 0..max_attempts {
            let start = Instant::now();

            let mut builder = self.client.get(&url);
            if let Some(q) = query {
                builder = builder.query(q);
            }
            if let Some(ref hdrs) = extra_headers {
                builder = builder.headers(hdrs.clone());
            }

            let result = builder.send().await;

            match result {
                Ok(response) => {
                    let elapsed = start.elapsed();
                    let status = response.status();
                    tracing::debug!(
                        url = %url,
                        status = %status.as_u16(),
                        elapsed_ms = %elapsed.as_millis(),
                        attempt = attempt + 1,
                        "External API request completed"
                    );

                    if Self::is_retryable_status(status) && attempt < self.retry_config.max_retries
                    {
                        let delay = self.retry_delay(attempt);
                        tracing::warn!(
                            url = %url,
                            status = %status.as_u16(),
                            attempt = attempt + 1,
                            delay_ms = %delay.as_millis(),
                            "Retrying request after server error"
                        );
                        tokio::time::sleep(delay).await;
                        last_error = Some(self.handle_error(response).await);
                        continue;
                    }

                    return self.handle_response(response).await;
                }
                Err(e) => {
                    let elapsed = start.elapsed();

                    if Self::is_retryable_error(&e) && attempt < self.retry_config.max_retries {
                        let delay = self.retry_delay(attempt);
                        tracing::warn!(
                            url = %url,
                            error = %e,
                            attempt = attempt + 1,
                            delay_ms = %delay.as_millis(),
                            elapsed_ms = %elapsed.as_millis(),
                            "Retrying request after error"
                        );
                        tokio::time::sleep(delay).await;
                        last_error = Some(AppError::Internal(e.into()));
                        continue;
                    }

                    tracing::error!(
                        url = %url,
                        error = %e,
                        elapsed_ms = %elapsed.as_millis(),
                        "External API request failed (no more retries)"
                    );
                    return Err(AppError::Internal(e.into()));
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| AppError::Server("Request failed after retries".to_string())))
    }

    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            response
                .json()
                .await
                .map_err(|e| AppError::Internal(e.into()))
        } else {
            Err(self.handle_error(response).await)
        }
    }

    async fn handle_error(&self, response: Response) -> AppError {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|e| format!("[failed to read response body: {}]", e));

        tracing::warn!(status = %status, body = %body, "API request failed");

        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                AppError::Unauthorized("Upstream authentication failed".to_string())
            }
            StatusCode::NOT_FOUND => AppError::NotFound("Resource not found".to_string()),
            StatusCode::TOO_MANY_REQUESTS => AppError::RateLimited,
            s if s.is_server_error() => {
                let truncated_body = if body.chars().count() > 200 {
                    format!("{}...", body.chars().take(200).collect::<String>())
                } else {
                    body
                };
                AppError::Server(format!(
                    "Upstream request failed with status {}: {}",
                    status, truncated_body
                ))
            }
            _ => AppError::Server(format!("Upstream request failed with status {}", status)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url() {
        let client =
            ApiClient::new("https://api.example.com", Duration::from_secs(30))
                .expect("Failed to create API client");

        assert_eq!(client.build_url("/users"), "https://api.example.com/users");
        assert_eq!(client.build_url("users"), "https://api.example.com/users");
    }

    #[test]
    fn test_build_url_with_trailing_slash() {
        let client =
            ApiClient::new("https://api.example.com/", Duration::from_secs(30))
                .expect("Failed to create API client");

        assert_eq!(client.build_url("/users"), "https://api.example.com/users");
    }
}
