//! GraphQL schema and handlers.

mod schema;
pub mod types;

pub use schema::{AppContext, AppSchema, QueryRoot, build_schema};
pub use types::{Artist, Track};
