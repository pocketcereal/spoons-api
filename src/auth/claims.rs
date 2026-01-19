//! JWT claims structures.

use serde::{Deserialize, Serialize};

/// Supabase JWT claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Audience
    #[serde(default)]
    pub aud: String,
    /// Expiration time
    pub exp: u64,
    /// Issued at
    #[serde(default)]
    pub iat: u64,
    /// Email (optional)
    #[serde(default)]
    pub email: Option<String>,
    /// Role (optional)
    #[serde(default)]
    pub role: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_deserialize() {
        let json =
            r#"{"sub": "user123", "aud": "authenticated", "exp": 9999999999, "iat": 1000000000}"#;
        let claims: Claims = serde_json::from_str(json).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.aud, "authenticated");
    }
}
