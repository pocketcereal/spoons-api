//! Server initialization and startup.

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::auth::AuthConfig;
use crate::config::AppConfig;
use crate::db::{DbConfig, create_pool};
use crate::error::{AppError, Result};
use crate::graphql::{AppContext, build_schema};
use crate::musicbrainz::MusicBrainzClient;
use crate::routes;

/// Run the HTTP server.
pub async fn run(config: &AppConfig) -> Result<()> {
    let auth_config = AuthConfig::from_env();

    tracing::info!(
        auth_enabled = auth_config.enabled,
        dev_mode = auth_config.is_dev_mode(),
        "Auth configuration loaded"
    );

    // Create database pool
    let db_url = config
        .database
        .url
        .clone()
        .or_else(|| std::env::var("SPOONS_DATABASE_URL").ok())
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .ok_or_else(|| AppError::Config("DATABASE_URL must be set".to_string()))?;

    let db_config = DbConfig {
        url: db_url,
        max_connections: config.database.max_connections,
    };

    let db_pool = create_pool(&db_config)?;
    tracing::info!(
        max_connections = config.database.max_connections,
        cache_ttl_seconds = config.database.cache_ttl_seconds,
        "Database pool initialized"
    );

    // Create MusicBrainz client and application context
    let musicbrainz_client = MusicBrainzClient::default_client()?;
    let app_context = AppContext {
        db_pool,
        musicbrainz_client,
        cache_ttl_seconds: config.database.cache_ttl_seconds,
    };

    let schema = build_schema(app_context);
    tracing::info!("GraphQL schema initialized with MusicBrainz client and database cache");

    // Build router with auth middleware on protected routes
    let app = routes::build_router(auth_config, schema).layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!(%addr, "Starting server");

    let listener = TcpListener::bind(addr).await.map_err(|e| {
        AppError::Server(format!("Failed to bind to {}: {}", addr, e))
    })?;

    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Server(format!("Server error: {}", e)))?;

    Ok(())
}
