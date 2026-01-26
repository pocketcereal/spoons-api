//! Authentication middleware for axum.

use axum::{
    Json,
    body::Body,
    extract::Request,
    http::{StatusCode, HeaderValue, header::{AUTHORIZATION, WWW_AUTHENTICATE}},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, jwk::JwkSet};
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::Claims;

/// Minimum interval between JWKS refresh attempts (1 minute).
const JWKS_REFRESH_MIN_INTERVAL: Duration = Duration::from_secs(60);

/// JWKS cache with on-demand refresh support.
#[derive(Debug)]
pub struct JwksCache {
    /// The cached JWKS.
    jwks: RwLock<Option<JwkSet>>,
    /// Last refresh timestamp for rate limiting.
    last_refresh: RwLock<Option<Instant>>,
    /// Supabase URL for fetching JWKS.
    supabase_url: Option<String>,
}

impl JwksCache {
    pub fn new(supabase_url: Option<String>) -> Self {
        Self {
            jwks: RwLock::new(None),
            last_refresh: RwLock::new(None),
            supabase_url,
        }
    }

    /// Get the cached JWKS.
    pub async fn get(&self) -> Option<JwkSet> {
        self.jwks.read().await.clone()
    }

    /// Set the JWKS cache.
    pub async fn set(&self, jwks: JwkSet) {
        *self.jwks.write().await = Some(jwks);
        *self.last_refresh.write().await = Some(Instant::now());
    }

    /// Check if refresh is allowed (rate limiting).
    async fn can_refresh(&self) -> bool {
        let last = self.last_refresh.read().await;
        match *last {
            Some(instant) => instant.elapsed() >= JWKS_REFRESH_MIN_INTERVAL,
            None => true,
        }
    }

    /// Attempt to refresh JWKS from Supabase.
    /// Returns true if refresh was attempted, false if rate limited.
    pub async fn try_refresh(&self) -> Result<bool, String> {
        let Some(ref url) = self.supabase_url else {
            return Ok(false);
        };

        if !self.can_refresh().await {
            tracing::debug!("JWKS refresh rate limited");
            return Ok(false);
        }

        let jwks_url = format!("{}/.well-known/jwks.json", url.trim_end_matches('/'));
        tracing::info!(url = %jwks_url, "Refreshing JWKS from Supabase");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
            .get(&jwks_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch JWKS: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("JWKS fetch failed with status: {}", response.status()));
        }

        let jwks: JwkSet = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JWKS: {}", e))?;

        tracing::info!(keys = jwks.keys.len(), "JWKS refreshed successfully");
        self.set(jwks).await;
        Ok(true)
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub supabase_url: Option<String>,
    pub jwks: Option<Arc<JwkSet>>,
    pub jwt_secret: Option<String>,
    pub dev_token: Option<String>,
    pub enabled: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            supabase_url: None,
            jwks: None,
            jwt_secret: None,
            dev_token: None,
            enabled: true,
        }
    }
}

impl AuthConfig {
    pub fn from_env() -> Self {
        #[cfg(debug_assertions)]
        let dev_token = std::env::var("SPOONS_DEV_TOKEN").ok();
        #[cfg(not(debug_assertions))]
        let dev_token: Option<String> = None;

        Self {
            supabase_url: std::env::var("SPOONS_SUPABASE_URL").ok(),
            jwks: None,
            jwt_secret: std::env::var("SPOONS_JWT_SECRET").ok(),
            dev_token,
            enabled: std::env::var("SPOONS_AUTH_DISABLED")
                .map(|v| v != "true" && v != "1")
                .unwrap_or(true),
        }
    }

    pub fn is_dev_mode(&self) -> bool {
        self.dev_token.is_some()
    }

    /// Fetches and caches JWKS from Supabase. Call at startup before creating auth layer.
    pub async fn fetch_jwks(&mut self) -> Result<(), String> {
        let Some(ref url) = self.supabase_url else {
            tracing::debug!("No Supabase URL configured, skipping JWKS fetch");
            return Ok(());
        };

        let jwks_url = format!("{}/.well-known/jwks.json", url.trim_end_matches('/'));
        tracing::info!(url = %jwks_url, "Fetching JWKS from Supabase");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
            .get(&jwks_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch JWKS: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("JWKS fetch failed with status: {}", response.status()));
        }

        let jwks: JwkSet = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JWKS: {}", e))?;

        tracing::info!(keys = jwks.keys.len(), "JWKS fetched successfully");
        self.jwks = Some(Arc::new(jwks));
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct AuthError {
    error: &'static str,
    message: String,
}

/// Creates an unauthorized response with WWW-Authenticate header per RFC 7235.
fn unauthorized_response(message: &str) -> Response {
    let mut response = (
        StatusCode::UNAUTHORIZED,
        Json(AuthError {
            error: "Unauthorized",
            message: message.to_string(),
        }),
    )
        .into_response();

    response.headers_mut().insert(
        WWW_AUTHENTICATE,
        HeaderValue::from_static("Bearer"),
    );

    response
}

fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("bearer "))
}

/// JWT validation leeway in seconds for clock skew tolerance.
const JWT_LEEWAY_SECONDS: u64 = 30;

/// Expected JWT audience for Supabase tokens.
const JWT_AUDIENCE: &str = "authenticated";

/// JWT validation error types.
#[derive(Debug)]
enum JwtError {
    /// Key ID not found in JWKS - may trigger refresh.
    KidNotFound(String),
    /// Other validation error - won't trigger refresh.
    Other(String),
}

impl std::fmt::Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JwtError::KidNotFound(kid) => write!(f, "Key '{}' not found in JWKS", kid),
            JwtError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

/// Validates JWT against JWKS.
fn validate_jwt_with_jwks(token: &str, jwks: &JwkSet) -> Result<Claims, JwtError> {
    let header = jsonwebtoken::decode_header(token)
        .map_err(|e| JwtError::Other(format!("Invalid token header: {}", e)))?;

    if header.alg != Algorithm::RS256 {
        return Err(JwtError::Other(format!(
            "Invalid token algorithm: expected RS256, got {:?}",
            header.alg
        )));
    }

    let kid = header
        .kid
        .ok_or_else(|| JwtError::Other("Token missing kid header".to_string()))?;

    let jwk = jwks.find(&kid).ok_or(JwtError::KidNotFound(kid))?;

    let decoding_key = DecodingKey::from_jwk(jwk)
        .map_err(|e| JwtError::Other(format!("Invalid JWK: {}", e)))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_required_spec_claims(&["sub", "exp"]);
    validation.set_audience(&[JWT_AUDIENCE]);
    validation.leeway = JWT_LEEWAY_SECONDS;

    decode::<Claims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| JwtError::Other(format!("Invalid token: {}", e)))
}

/// Validates JWT with HS256 secret.
fn validate_jwt_with_secret(token: &str, secret: &str) -> Result<Claims, String> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["sub", "exp"]);
    validation.set_audience(&[JWT_AUDIENCE]);
    validation.leeway = JWT_LEEWAY_SECONDS;

    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

    decode::<Claims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| format!("Invalid token: {}", e))
}

/// Tries RS256 with JWKS first, falls back to HS256 with JWT secret.
fn validate_jwt(token: &str, config: &AuthConfig) -> Result<Claims, String> {
    if let Some(ref jwks) = config.jwks {
        return validate_jwt_with_jwks(token, jwks).map_err(|e| e.to_string());
    }

    let Some(ref secret) = config.jwt_secret else {
        return Err("No JWKS or JWT secret configured".to_string());
    };

    validate_jwt_with_secret(token, secret)
}

fn is_dev_token_match(token: &str, config: &AuthConfig) -> bool {
    config
        .dev_token
        .as_ref()
        .is_some_and(|dev_token| constant_time_eq(token.as_bytes(), dev_token.as_bytes()))
}

/// Constant-time comparison to prevent timing attacks on token validation.
/// Uses fixed iteration count based on max length to avoid leaking length info.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());
    let mut result = (a.len() != b.len()) as u8;

    for i in 0..max_len {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        result |= x ^ y;
    }

    result == 0
}

/// Creates an auth layer with JWKS on-demand refresh support.
pub fn auth_layer(config: AuthConfig) -> AuthLayer {
    let jwks_cache = Arc::new(JwksCache::new(config.supabase_url.clone()));

    if let Some(ref jwks) = config.jwks {
        let cache = jwks_cache.clone();
        let jwks = (**jwks).clone();
        tokio::spawn(async move {
            cache.set(jwks).await;
        });
    }

    AuthLayer {
        config: Arc::new(config),
        jwks_cache,
    }
}

#[derive(Clone)]
pub struct AuthLayer {
    config: Arc<AuthConfig>,
    jwks_cache: Arc<JwksCache>,
}

impl<S> tower::Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            config: self.config.clone(),
            jwks_cache: self.jwks_cache.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    config: Arc<AuthConfig>,
    jwks_cache: Arc<JwksCache>,
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

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let config = self.config.clone();
        let jwks_cache = self.jwks_cache.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            if !config.enabled {
                return inner.call(req).await;
            }

            let auth_header = req
                .headers()
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok());

            let Some(auth_header) = auth_header else {
                return Ok(unauthorized_response("Missing Authorization header"));
            };

            let Some(token) = extract_bearer_token(auth_header) else {
                return Ok(unauthorized_response("Invalid Authorization header format"));
            };

            if is_dev_token_match(token, &config) {
                tracing::warn!("Dev token authentication used - ensure this is disabled in production");
                return inner.call(req).await;
            }

            if let Some(jwks) = jwks_cache.get().await {
                match validate_jwt_with_jwks(token, &jwks) {
                    Ok(claims) => {
                        tracing::debug!(sub = %claims.sub, "JWT authentication successful");
                        req.extensions_mut().insert(claims);
                        return inner.call(req).await;
                    }
                    Err(JwtError::KidNotFound(kid)) => {
                        tracing::info!(kid = %kid, "Key not found, attempting JWKS refresh");
                        if jwks_cache.try_refresh().await.unwrap_or(false)
                            && let Some(refreshed_jwks) = jwks_cache.get().await
                            && let Ok(claims) = validate_jwt_with_jwks(token, &refreshed_jwks)
                        {
                            tracing::debug!(sub = %claims.sub, "JWT authentication successful after JWKS refresh");
                            req.extensions_mut().insert(claims);
                            return inner.call(req).await;
                        }
                        tracing::warn!(kid = %kid, "JWT authentication failed: key not found even after refresh");
                        return Ok(unauthorized_response("Invalid or expired token"));
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "JWT authentication failed");
                        return Ok(unauthorized_response("Invalid or expired token"));
                    }
                }
            }

            match validate_jwt(token, &config) {
                Ok(claims) => {
                    tracing::debug!(sub = %claims.sub, "JWT authentication successful");
                    req.extensions_mut().insert(claims);
                    inner.call(req).await
                }
                Err(err) => {
                    tracing::warn!(error = %err, "JWT authentication failed");
                    Ok(unauthorized_response("Invalid or expired token"))
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
