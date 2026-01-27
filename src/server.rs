//! Server initialization and startup.

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::audius::AudiusClient;
use crate::auth::AuthConfig;
use crate::config::AppConfig;
use crate::db::{DbConfig, create_pool};
use crate::error::{AppError, Result};
use crate::graphql::{AppContext, build_schema};
use crate::musicbrainz::MusicBrainzClient;
use crate::podcast_index::PodcastIndexClient;
use crate::routes;

pub async fn run(config: &AppConfig) -> Result<()> {
    let mut auth_config = AuthConfig::from_env();

    if let Err(e) = auth_config.fetch_jwks().await {
        tracing::warn!(error = %e, "Failed to fetch JWKS, falling back to JWT secret");
    }

    // H8: Fail fast if auth is enabled but no valid auth method is configured
    if auth_config.enabled {
        let has_jwks = auth_config.jwks.is_some();
        let has_secret = auth_config.jwt_secret.is_some();
        let has_dev_token = auth_config.dev_token.is_some();

        if !has_jwks && !has_secret && !has_dev_token {
            return Err(AppError::Config(
                "Auth is enabled but no authentication method is configured. \
                Set SPOONS_SUPABASE_URL (for JWKS), SPOONS_JWT_SECRET, or SPOONS_DEV_TOKEN"
                    .to_string(),
            ));
        }

        // M9: Warn if only dev token is configured (already logs on each request, but also at startup)
        if has_dev_token && !has_jwks && !has_secret {
            tracing::warn!(
                "Only dev token authentication is configured - \
                ensure SPOONS_SUPABASE_URL or SPOONS_JWT_SECRET is set for production"
            );
        }
    }

    tracing::info!(
        auth_enabled = auth_config.enabled,
        dev_mode = auth_config.is_dev_mode(),
        jwks_loaded = auth_config.jwks.is_some(),
        "Auth configuration loaded"
    );

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

    let musicbrainz_client = MusicBrainzClient::default_client()?;

    let audius_client = if config.audius.enabled {
        match AudiusClient::new(&config.audius.app_name).await {
            Ok(client) => {
                tracing::info!(app_name = %config.audius.app_name, "Audius client initialized");
                Some(client)
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to initialize Audius client, Audius search will be disabled");
                None
            }
        }
    } else {
        tracing::info!("Audius integration disabled");
        None
    };

    let podcast_index_client = if config.podcast_index.enabled {
        match (
            &config.podcast_index.api_key,
            &config.podcast_index.api_secret,
        ) {
            (Some(api_key), Some(api_secret)) => {
                match PodcastIndexClient::with_base_url(
                    api_key,
                    api_secret,
                    &config.podcast_index.base_url,
                ) {
                    Ok(client) => {
                        tracing::info!(base_url = %config.podcast_index.base_url, "PodcastIndex client initialized");
                        Some(client)
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to initialize PodcastIndex client, podcast search will be disabled");
                        None
                    }
                }
            }
            _ => {
                tracing::warn!("PodcastIndex enabled but API credentials not configured");
                None
            }
        }
    } else {
        tracing::info!("PodcastIndex integration disabled");
        None
    };

    let app_context = AppContext {
        db_pool,
        musicbrainz_client,
        audius_client,
        podcast_index_client,
        cache_ttl_seconds: config.database.cache_ttl_seconds,
    };

    let schema = build_schema(app_context);
    tracing::info!("GraphQL schema initialized");

    let app = routes::build_router(auth_config, schema).layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!(%addr, "Starting server");

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Server(format!("Failed to bind to {}: {}", addr, e)))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Server(format!("Server error: {}", e)))?;

    Ok(())
}
