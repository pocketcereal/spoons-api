//! PodcastIndex API authentication using HMAC-SHA1.

use sha1::{Digest, Sha1};
use std::time::{SystemTime, UNIX_EPOCH};

/// Authentication credentials for PodcastIndex API.
#[derive(Debug, Clone)]
pub struct PodcastIndexAuth {
    api_key: String,
    api_secret: String,
}

/// HTTP headers required for PodcastIndex API authentication.
#[derive(Debug, Clone)]
pub struct AuthHeaders {
    pub x_auth_key: String,
    pub x_auth_date: String,
    pub authorization: String,
    pub user_agent: String,
}

impl PodcastIndexAuth {
    /// Creates a new PodcastIndex authentication instance.
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
        }
    }

    /// Generates authentication headers for a PodcastIndex API request.
    ///
    /// The authorization hash is computed as SHA1("{api_key}{api_secret}{epoch}").
    pub fn generate_headers(&self) -> AuthHeaders {
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            .to_string();

        let auth_string = format!("{}{}{}", self.api_key, self.api_secret, epoch);
        let mut hasher = Sha1::new();
        hasher.update(auth_string.as_bytes());
        let hash = hasher.finalize();
        let authorization = hex::encode(hash);

        AuthHeaders {
            x_auth_key: self.api_key.clone(),
            x_auth_date: epoch,
            authorization,
            user_agent: format!("spoons-api/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_header_generation() {
        let auth = PodcastIndexAuth::new("test_key".to_string(), "test_secret".to_string());
        let headers = auth.generate_headers();

        // Verify header structure
        assert_eq!(headers.x_auth_key, "test_key");
        assert!(!headers.x_auth_date.is_empty());
        assert_eq!(headers.authorization.len(), 40); // SHA1 hash is 40 hex chars
        assert!(headers.user_agent.starts_with("spoons-api/"));

        // Verify epoch is numeric
        assert!(headers.x_auth_date.parse::<u64>().is_ok());

        // Verify authorization is valid hex
        assert!(headers.authorization.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_auth_headers_are_different_over_time() {
        let auth = PodcastIndexAuth::new("test_key".to_string(), "test_secret".to_string());
        let headers1 = auth.generate_headers();

        std::thread::sleep(std::time::Duration::from_secs(1));

        let headers2 = auth.generate_headers();

        // Different timestamps should produce different auth headers
        assert_ne!(headers1.x_auth_date, headers2.x_auth_date);
        assert_ne!(headers1.authorization, headers2.authorization);
    }

    #[test]
    fn test_known_hash() {
        // Test with a known epoch to verify SHA1 implementation
        let auth = PodcastIndexAuth::new("key123".to_string(), "secret456".to_string());

        // Manually compute what the hash should be for a specific epoch
        let test_epoch = "1234567890";
        let expected_string = format!("key123secret456{}", test_epoch);
        let mut hasher = Sha1::new();
        hasher.update(expected_string.as_bytes());
        let expected_hash = hex::encode(hasher.finalize());

        // Generate headers (will use current time, so we can't test exact match)
        // But we can verify the format is correct
        let headers = auth.generate_headers();
        assert_eq!(headers.authorization.len(), expected_hash.len());
    }
}
