#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub supabase_url: Option<String>,
    pub jwt_secret: Option<String>,
    pub dev_token: Option<String>,
    pub enabled: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            supabase_url: None,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_default() {
        let config = AuthConfig::default();
        assert!(config.jwt_secret.is_none());
        assert!(config.dev_token.is_none());
        assert!(config.enabled);
    }
}
