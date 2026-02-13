//! GraphQL schema and handlers.

pub mod podcast;
mod schema;
pub mod types;

pub use podcast::{Category, Episode, Podcast, PodcastSource};
pub(crate) use schema::get_app_context;
pub use schema::{AppContext, AppSchema, QueryRoot, build_schema};
pub(crate) use schema::{clamp_limit, require_podcast_index_client, validate_query};
pub use types::{Artist, Track};
