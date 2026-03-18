pub mod graphql;
pub mod health;

use axum::Router;
use jsonwebtoken::jwk::JwkSet;

use crate::auth::{AuthConfig, auth_layer};
use crate::graphql::AppSchema;

pub async fn build_router(
    auth_config: AuthConfig,
    initial_jwks: Option<JwkSet>,
    schema: AppSchema,
) -> Router {
    let mut public_routes = Router::new().merge(health::routes());

    if !auth_config.enabled {
        tracing::info!("GraphiQL playground enabled at /graphiql (auth disabled)");
        public_routes = public_routes.merge(graphql::graphiql_route());
    }

    let protected_routes = Router::new()
        .merge(graphql::graphql_route(schema))
        .layer(auth_layer(auth_config, initial_jwks).await);

    Router::new().merge(public_routes).merge(protected_routes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphql::{AppContext, build_schema};

    #[tokio::test]
    async fn test_router_builds() {
        let auth_config = AuthConfig::default();

        let app_context = AppContext {
            music_providers: vec![],
            podcast_providers: vec![],
            audiobook_providers: vec![],
        };

        let schema = build_schema(app_context);
        let _ = build_router(auth_config, None, schema).await;
    }
}
