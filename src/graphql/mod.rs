//! GraphQL schema and handlers.

pub mod podcast;
mod schema;
pub mod types;

pub use podcast::{Category, Episode, Podcast, PodcastSource};
pub use schema::{AppContext, AppSchema, QueryRoot, build_schema};
pub use types::{Artist, Track};
