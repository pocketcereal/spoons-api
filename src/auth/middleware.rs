//! Authentication middleware for axum.

use axum::{
    Json,
    body::Body,
    extract::Request,
    http::{StatusCode, header::AUTHORIZATION},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Serialize;
use std::sync::Arc;

use super::Claims;

/// Authentication configuration.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// JWT secret for HS256 validation (used when jwt_secret is set)
    pub jwt_secret: Option<String>,
    /// Dev token for local development bypass
    pub dev_token: Option<String>,
    /// Whether auth is enabled
    pub enabled: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: None,
            dev_token: None,
            enabled: true,
        }
    }
}

impl AuthConfig {
    /// Create config from environment variables.
    pub fn from_env() -> Self {
        Self {
            jwt_secret: std::env::var("SPOONS_JWT_SECRET").ok(),
            dev_token: std::env::var("SPOONS_DEV_TOKEN").ok(),
            enabled: std::env::var("SPOONS_AUTH_DISABLED")
                .map(|v| v != "true" && v != "1")
                .unwrap_or(true),
        }
    }

    /// Check if dev mode is enabled.
    pub fn is_dev_mode(&self) -> bool {
        self.dev_token.is_some()
    }
}

/// Error response for auth failures.
#[derive(Debug, Serialize)]
struct AuthError {
    error: &'static str,
    message: String,
}

/// Extract Bearer token from Authorization header.
fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("bearer "))
}

/// Validate a JWT token and return claims.
fn validate_jwt(token: &str, config: &AuthConfig) -> Result<Claims, String> {
    let Some(ref secret) = config.jwt_secret else {
        return Err("JWT secret not configured".to_string());
    };

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["sub", "exp"]);
    validation.leeway = 60;

    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

    decode::<Claims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| format!("Invalid token: {}", e))
}

/// Check if token matches dev token for bypass.
fn is_dev_token_match(token: &str, config: &AuthConfig) -> bool {
    config
        .dev_token
        .as_ref()
        .is_some_and(|dev_token| token == dev_token)
}

/// Create the auth middleware layer.
pub fn auth_layer(config: AuthConfig) -> AuthLayer {
    AuthLayer {
        config: Arc::new(config),
    }
}

/// Auth middleware layer.
#[derive(Clone)]
pub struct AuthLayer {
    config: Arc<AuthConfig>,
}

impl<S> tower::Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

/// Auth middleware service.
#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    config: Arc<AuthConfig>,
}

impl<S> tower::Service<Request<Body>> for AuthMiddleware<S>
where
    S: tower::Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Skip auth if disabled
            if !config.enabled {
                return inner.call(req).await;
            }

            // Extract Authorization header
            let auth_header = req
                .headers()
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok());

            let Some(auth_header) = auth_header else {
                let response = (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthError {
                        error: "Unauthorized",
                        message: "Missing Authorization header".to_string(),
                    }),
                )
                    .into_response();
                return Ok(response);
            };

            let Some(token) = extract_bearer_token(auth_header) else {
                let response = (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthError {
                        error: "Unauthorized",
                        message: "Invalid Authorization header format".to_string(),
                    }),
                )
                    .into_response();
                return Ok(response);
            };

            // Check for dev token bypass
            if is_dev_token_match(token, &config) {
                tracing::debug!("Dev token authentication successful");
                return inner.call(req).await;
            }

            // Validate JWT
            match validate_jwt(token, &config) {
                Ok(claims) => {
                    tracing::debug!(sub = %claims.sub, "JWT authentication successful");
                    inner.call(req).await
                }
                Err(err) => {
                    tracing::warn!(error = %err, "JWT authentication failed");
                    let response = (
                        StatusCode::UNAUTHORIZED,
                        Json(AuthError {
                            error: "Unauthorized",
                            message: err,
                        }),
                    )
                        .into_response();
                    Ok(response)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(extract_bearer_token("Bearer abc123"), Some("abc123"));
        assert_eq!(extract_bearer_token("bearer xyz789"), Some("xyz789"));
        assert_eq!(extract_bearer_token("Basic abc123"), None);
    }

    #[test]
    fn test_auth_config_default() {
        let config = AuthConfig::default();
        assert!(config.jwt_secret.is_none());
        assert!(config.dev_token.is_none());
        assert!(config.enabled);
    }

    #[test]
    fn test_is_dev_token_match() {
        let config = AuthConfig {
            dev_token: Some("test-token".to_string()),
            ..Default::default()
        };
        assert!(is_dev_token_match("test-token", &config));
        assert!(!is_dev_token_match("wrong-token", &config));

        let config_no_token = AuthConfig::default();
        assert!(!is_dev_token_match("any-token", &config_no_token));
    }
}
