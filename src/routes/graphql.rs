use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    Router,
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
};

use crate::graphql::AppSchema;

async fn graphql_handler(State(schema): State<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql_handler() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub fn graphql_route(schema: AppSchema) -> Router {
    Router::new()
        .route("/graphql", post(graphql_handler).get(graphql_handler))
        .with_state(schema)
}

pub fn graphiql_route() -> Router {
    Router::new().route("/graphiql", get(graphiql_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{DbConfig, create_pool};
    use crate::graphql::{AppContext, build_schema};
    use crate::musicbrainz::MusicBrainzClient;
    use crate::services::MusicService;

    #[test]
    fn test_routes_build() {
        let client = MusicBrainzClient::new("https://musicbrainz.org/ws/2")
            .expect("Failed to create MusicBrainz client");

        let db_url = match std::env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => return,
        };

        let db_config = DbConfig {
            url: db_url,
            max_connections: 1,
        };

        let pool = create_pool(&db_config).expect("Failed to create database pool");

        let app_context = AppContext {
            music: MusicService::new(pool, client, None, 3600),
            podcast: None,
            audiobook: None,
        };

        let schema = build_schema(app_context);
        let _ = graphql_route(schema);
    }
}
