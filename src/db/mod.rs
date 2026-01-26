//! Database module for PostgreSQL with diesel-async.

mod helpers;
mod pool;
pub mod models;
pub mod repositories;
pub mod schema;

pub use helpers::{db_error, get_conn, parse_uuid, spawn_cache_task};
pub use pool::{DbConfig, DbPool, create_pool};
pub use repositories::MusicRepository;
