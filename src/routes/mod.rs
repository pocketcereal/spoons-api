//! API route definitions.

pub mod graphql;
pub mod health;

use axum::Router;

use crate::auth::{AuthConfig, auth_layer};
use crate::graphql::AppSchema;

pub fn build_router(auth_config: AuthConfig, schema: AppSchema) -> Router {
    let mut public_routes = Router::new().merge(health::routes());

    if !auth_config.enabled {
        tracing::info!("GraphiQL playground enabled at /graphiql (auth disabled)");
        public_routes = public_routes.merge(graphql::graphiql_route());
    }

    let protected_routes = Router::new()
        .merge(graphql::graphql_route(schema))
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
            audius_client: None,
            cache_ttl_seconds: 3600,
        };

        let schema = build_schema(app_context);
        let _ = build_router(auth_config, schema);
    }
}
