//! API route definitions.

pub mod graphql;
pub mod health;

use axum::Router;

use crate::auth::{AuthConfig, auth_layer};
use crate::graphql::AppSchema;

/// Build the application router with all routes.
pub fn build_router(auth_config: AuthConfig, schema: AppSchema) -> Router {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .merge(health::routes())
        .merge(graphql::routes(schema));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Future protected routes will be added here
        .layer(auth_layer(auth_config));

    Router::new().merge(public_routes).merge(protected_routes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{DbConfig, create_pool};
    use crate::graphql::{AppContext, build_schema};
    use crate::musicbrainz::MusicBrainzClient;

    #[test]
    fn test_router_builds() {
        let auth_config = AuthConfig::default();
        let client = MusicBrainzClient::new("https://musicbrainz.org/ws/2").unwrap();

        // Skip test if no DATABASE_URL - this is expected in CI without DB
        let db_url = match std::env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => return,
        };

        let db_config = DbConfig {
            url: db_url,
            max_connections: 1,
        };

        let pool = create_pool(&db_config).unwrap();

        let app_context = AppContext {
            db_pool: pool,
            musicbrainz_client: client,
            cache_ttl_seconds: 3600,
        };

        let schema = build_schema(app_context);
        let _ = build_router(auth_config, schema);
    }
}
