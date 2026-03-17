use jsonwebtoken::jwk::JwkSet;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Minimum interval between JWKS refresh attempts (1 minute).
const JWKS_REFRESH_MIN_INTERVAL: Duration = Duration::from_secs(60);

fn new_http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

/// Fetch JWKS from a Supabase URL.
///
/// Expects `supabase_url` to be the project root (e.g., `https://xxx.supabase.co`),
/// not the auth endpoint (`https://xxx.supabase.co/auth/v1`).
async fn fetch_jwks_from_url(
    client: &reqwest::Client,
    supabase_url: &str,
) -> Result<JwkSet, String> {
    let jwks_url = format!(
        "{}/auth/v1/.well-known/jwks.json",
        supabase_url.trim_end_matches('/')
    );
    tracing::info!(url = %jwks_url, "Fetching JWKS from Supabase");

    let response = client
        .get(&jwks_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch JWKS: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "JWKS fetch failed with status: {}",
            response.status()
        ));
    }

    let jwks: JwkSet = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JWKS: {}", e))?;

    tracing::info!(keys = jwks.keys.len(), "JWKS fetched successfully");
    Ok(jwks)
}

/// Fetch JWKS from Supabase at startup. Returns None if no URL is configured.
pub async fn fetch_jwks(supabase_url: Option<&str>) -> Result<Option<JwkSet>, String> {
    let Some(url) = supabase_url else {
        tracing::debug!("No Supabase URL configured, skipping JWKS fetch");
        return Ok(None);
    };

    let client = new_http_client()?;
    let jwks = fetch_jwks_from_url(&client, url).await?;
    Ok(Some(jwks))
}

#[derive(Debug)]
pub struct JwksCache {
    jwks: RwLock<Option<JwkSet>>,
    /// For rate-limiting refresh attempts.
    last_refresh: RwLock<Option<Instant>>,
    supabase_url: Option<String>,
    http_client: Option<reqwest::Client>,
}

impl JwksCache {
    pub fn new(supabase_url: Option<String>) -> Self {
        let http_client = supabase_url.as_ref().and_then(|_| {
            new_http_client()
                .inspect_err(|e| tracing::warn!(error = %e, "Failed to create JWKS HTTP client"))
                .ok()
        });
        Self {
            jwks: RwLock::new(None),
            last_refresh: RwLock::new(None),
            supabase_url,
            http_client,
        }
    }

    pub async fn get(&self) -> Option<JwkSet> {
        self.jwks.read().await.clone()
    }

    pub async fn set(&self, jwks: JwkSet) {
        *self.jwks.write().await = Some(jwks);
        *self.last_refresh.write().await = Some(Instant::now());
    }

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
        let Some(ref client) = self.http_client else {
            return Err("No HTTP client available for JWKS refresh".to_string());
        };

        if !self.can_refresh().await {
            tracing::debug!("JWKS refresh rate limited");
            return Ok(false);
        }

        let jwks = fetch_jwks_from_url(client, url).await?;
        self.set(jwks).await;
        Ok(true)
    }
}
