//! Health check endpoint.

use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};

/// Health check response.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Health check handler.
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Build the health routes.
pub fn routes() -> Router {
    Router::new().route("/healthz", get(health_check))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_health_check() {
        let app = routes();
        let server = TestServer::new(app).unwrap();

        let response = server.get("/healthz").await;
        response.assert_status_ok();

        let body: HealthResponse = response.json();
        assert_eq!(body.status, "ok");
        assert!(!body.version.is_empty());
    }
}
