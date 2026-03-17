use axum::{
    Json,
    body::Body,
    extract::Request,
    http::{
        HeaderValue, StatusCode,
        header::{AUTHORIZATION, WWW_AUTHENTICATE},
    },
    response::{IntoResponse, Response},
};
use jsonwebtoken::jwk::JwkSet;
use serde::Serialize;
use std::sync::Arc;

use super::Claims;
use super::config::AuthConfig;
use super::jwks::JwksCache;
use super::validation::{
    JwtError, is_dev_token_match, validate_jwt_with_jwks, validate_jwt_with_secret_fallback,
};

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

    response
        .headers_mut()
        .insert(WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));

    response
}

fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    if auth_header.get(..7)?.eq_ignore_ascii_case("bearer ") {
        Some(&auth_header[7..])
    } else {
        None
    }
}

pub async fn auth_layer(config: AuthConfig, initial_jwks: Option<JwkSet>) -> AuthLayer {
    let jwks_cache = Arc::new(JwksCache::new(config.supabase_url.clone()));

    if let Some(jwks) = initial_jwks {
        jwks_cache.set(jwks).await;
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
        let inner_clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner_clone);

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
                tracing::warn!(
                    "Dev token authentication used - ensure this is disabled in production"
                );
                let dev_claims = Claims {
                    sub: "dev-user".to_string(),
                    aud: "authenticated".to_string(),
                    exp: u64::MAX,
                    iat: 0,
                    email: Some("dev@localhost".to_string()),
                    role: Some("authenticated".to_string()),
                };
                req.extensions_mut().insert(dev_claims);
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

            match validate_jwt_with_secret_fallback(token, &config) {
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
}
