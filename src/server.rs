use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::audius::AudiusClient;
use crate::auth::{AuthConfig, fetch_jwks};
use crate::config::AppConfig;
use crate::db::{DbConfig, create_pool};
use crate::error::{AppError, Result};
use crate::graphql::{AppContext, build_schema};
use crate::musicbrainz::MusicBrainzClient;
use crate::podcast_index::PodcastIndexClient;
use crate::routes;
use crate::services::{MusicService, PodcastService};

pub async fn run(config: &AppConfig) -> Result<()> {
    let auth_config = AuthConfig::from_env();

    let initial_jwks = match fetch_jwks(auth_config.supabase_url.as_deref()).await {
        Ok(jwks) => jwks,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to fetch JWKS, falling back to JWT secret");
            None
        }
    };

    if auth_config.enabled {
        let has_jwks = initial_jwks.is_some();
        let has_secret = auth_config.jwt_secret.is_some();
        let has_dev_token = auth_config.dev_token.is_some();

        if !has_jwks && !has_secret && !has_dev_token {
            return Err(AppError::Config(
                "Auth is enabled but no authentication method is configured. \
                Set SPOONS_SUPABASE_URL (for JWKS), SPOONS_JWT_SECRET, or SPOONS_DEV_TOKEN"
                    .to_string(),
            ));
        }

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
        jwks_loaded = initial_jwks.is_some(),
        "Auth configuration loaded"
    );

    let db_config = DbConfig::try_from(&config.database)?;
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

    let music = MusicService::new(
        db_pool.clone(),
        musicbrainz_client,
        audius_client,
        config.database.cache_ttl_seconds,
    );

    let podcast = if config.podcast_index.enabled {
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
                        Some(PodcastService::new(
                            db_pool,
                            client,
                            config.database.cache_ttl_seconds,
                        ))
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

    let app_context = AppContext { music, podcast };

    let schema = build_schema(app_context);
    tracing::info!("GraphQL schema initialized");

    let app = routes::build_router(auth_config, initial_jwks, schema)
        .await
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!(%addr, "Starting server");

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Server(format!("Failed to bind to {}: {}", addr, e)))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| AppError::Server(format!("Server error: {}", e)))?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Received Ctrl+C, shutting down"),
        _ = terminate => tracing::info!("Received SIGTERM, shutting down"),
    }
}
