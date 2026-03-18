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
    use crate::graphql::{AppContext, build_schema};

    #[test]
    fn test_routes_build() {
        let app_context = AppContext {
            music_providers: vec![],
            podcast_providers: vec![],
            audiobook_providers: vec![],
        };

        let schema = build_schema(app_context);
        let _ = graphql_route(schema);
    }
}
