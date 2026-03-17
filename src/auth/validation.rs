use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, jwk::JwkSet};

use super::Claims;
use super::config::AuthConfig;

/// JWT validation leeway in seconds for clock skew tolerance.
const JWT_LEEWAY_SECONDS: u64 = 30;

/// Expected JWT audience for Supabase tokens.
const JWT_AUDIENCE: &str = "authenticated";

const ALLOWED_JWKS_ALGORITHMS: &[Algorithm] = &[Algorithm::RS256, Algorithm::ES256];

#[derive(Debug)]
pub(crate) enum JwtError {
    /// May trigger JWKS refresh.
    KidNotFound(String),
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

pub(crate) fn validate_jwt_with_jwks(token: &str, jwks: &JwkSet) -> Result<Claims, JwtError> {
    let header = jsonwebtoken::decode_header(token)
        .map_err(|e| JwtError::Other(format!("Invalid token header: {}", e)))?;

    if !ALLOWED_JWKS_ALGORITHMS.contains(&header.alg) {
        return Err(JwtError::Other(format!(
            "Unsupported token algorithm: {:?}",
            header.alg
        )));
    }

    let kid = header
        .kid
        .ok_or_else(|| JwtError::Other("Token missing kid header".to_string()))?;

    let jwk = jwks.find(&kid).ok_or(JwtError::KidNotFound(kid))?;

    let decoding_key =
        DecodingKey::from_jwk(jwk).map_err(|e| JwtError::Other(format!("Invalid JWK: {}", e)))?;

    let mut validation = Validation::new(header.alg);
    validation.set_required_spec_claims(&["sub", "exp"]);
    validation.set_audience(&[JWT_AUDIENCE]);
    validation.leeway = JWT_LEEWAY_SECONDS;

    decode::<Claims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| JwtError::Other(format!("Invalid token: {}", e)))
}

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

/// HS256 fallback when JWKS cache is empty.
pub(crate) fn validate_jwt_with_secret_fallback(
    token: &str,
    config: &AuthConfig,
) -> Result<Claims, String> {
    let Some(ref secret) = config.jwt_secret else {
        return Err("No JWKS or JWT secret configured".to_string());
    };

    validate_jwt_with_secret(token, secret)
}

pub(crate) fn is_dev_token_match(token: &str, config: &AuthConfig) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

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
