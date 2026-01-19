//! Database module for PostgreSQL with diesel-async.

mod pool;
pub mod models;
pub mod repositories;
pub mod schema;

pub use pool::{DbConfig, DbPool, create_pool};
pub use repositories::MusicRepository;
