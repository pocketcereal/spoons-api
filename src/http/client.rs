//! API client wrapper with common functionality.

#![allow(dead_code)]

use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::time::Duration;

use crate::error::{AppError, Result};

/// Authentication method for API requests.
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// Bearer token authentication.
    Bearer(String),
    /// API key in header.
    ApiKey { header: String, key: String },
    /// Basic authentication.
    Basic { username: String, password: String },
    /// No authentication.
    None,
}

/// API client for making HTTP requests with shared configuration.
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    auth: AuthMethod,
    user_agent: String,
}

impl ApiClient {
    /// Create a new API client.
    pub fn new(base_url: impl Into<String>, auth: AuthMethod, timeout: Duration) -> Result<Self> {
        let user_agent = format!("spoons-api/{}", env!("CARGO_PKG_VERSION"));

        let client = Client::builder()
            .timeout(timeout)
            .user_agent(&user_agent)
            .build()
            .map_err(|e| AppError::Internal(e.into()))?;

        Ok(Self {
            client,
            base_url: base_url.into(),
            auth,
            user_agent,
        })
    }

    /// Build a full URL from a path.
    fn build_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{}/{}", base, path)
    }

    /// Add authentication to a request builder.
    fn add_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.auth {
            AuthMethod::Bearer(token) => builder.bearer_auth(token),
            AuthMethod::ApiKey { header, key } => builder.header(header.as_str(), key.as_str()),
            AuthMethod::Basic { username, password } => {
                builder.basic_auth(username, Some(password))
            }
            AuthMethod::None => builder,
        }
    }

    /// Make a GET request.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.build_url(path);
        tracing::debug!(url = %url, "Making GET request");

        let builder = self.client.get(&url);
        let builder = self.add_auth(builder);

        let response = builder
            .send()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        self.handle_response(response).await
    }

    /// Make a GET request with query parameters.
    pub async fn get_with_query<T: DeserializeOwned, Q: serde::Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        let url = self.build_url(path);
        tracing::debug!(url = %url, "Making GET request with query");

        let builder = self.client.get(&url).query(query);
        let builder = self.add_auth(builder);

        let response = builder
            .send()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        self.handle_response(response).await
    }

    /// Make a POST request.
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.build_url(path);
        tracing::debug!(url = %url, "Making POST request");

        let builder = self.client.post(&url).json(body);
        let builder = self.add_auth(builder);

        let response = builder
            .send()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        self.handle_response(response).await
    }

    /// Make a PUT request.
    pub async fn put<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.build_url(path);
        tracing::debug!(url = %url, "Making PUT request");

        let builder = self.client.put(&url).json(body);
        let builder = self.add_auth(builder);

        let response = builder
            .send()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        self.handle_response(response).await
    }

    /// Make a DELETE request.
    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = self.build_url(path);
        tracing::debug!(url = %url, "Making DELETE request");

        let builder = self.client.delete(&url);
        let builder = self.add_auth(builder);

        let response = builder
            .send()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(self.handle_error(response).await)
        }
    }

    /// Handle a successful response.
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

    /// Handle an error response.
    async fn handle_error(&self, response: Response) -> AppError {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        tracing::warn!(status = %status, body = %body, "API request failed");

        match status {
            StatusCode::UNAUTHORIZED => AppError::Server("Unauthorized".to_string()),
            StatusCode::FORBIDDEN => AppError::Server("Forbidden".to_string()),
            StatusCode::NOT_FOUND => AppError::Server("Not found".to_string()),
            StatusCode::TOO_MANY_REQUESTS => AppError::Server("Rate limited".to_string()),
            _ => AppError::Server(format!("Request failed with status {}: {}", status, body)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url() {
        let client = ApiClient::new(
            "https://api.example.com",
            AuthMethod::None,
            Duration::from_secs(30),
        )
        .unwrap();

        assert_eq!(client.build_url("/users"), "https://api.example.com/users");
        assert_eq!(client.build_url("users"), "https://api.example.com/users");
    }

    #[test]
    fn test_build_url_with_trailing_slash() {
        let client = ApiClient::new(
            "https://api.example.com/",
            AuthMethod::None,
            Duration::from_secs(30),
        )
        .unwrap();

        assert_eq!(client.build_url("/users"), "https://api.example.com/users");
    }
}
