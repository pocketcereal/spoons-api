//! GraphQL route handlers.

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Router,
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
};

use crate::graphql::AppSchema;

/// GraphQL query handler.
async fn graphql_handler(State(schema): State<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// GraphiQL playground handler.
async fn graphiql_handler() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

/// Build the GraphQL routes.
pub fn routes(schema: AppSchema) -> Router {
    Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphiql", get(graphiql_handler))
        .with_state(schema)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{DbConfig, create_pool};
    use crate::graphql::{AppContext, build_schema};
    use crate::musicbrainz::MusicBrainzClient;

    #[test]
    fn test_routes_build() {
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
        let _ = routes(schema);
    }
}
