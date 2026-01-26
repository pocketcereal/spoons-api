//! Database module for PostgreSQL with diesel-async.

mod helpers;
mod pool;
pub mod models;
pub mod repositories;
pub mod schema;

pub use helpers::{db_error, get_conn, parse_uuid, spawn_cache_task, validate_batch_size, MAX_BATCH_SIZE};
pub use pool::{DbConfig, DbPool, create_pool};
pub use repositories::MusicRepository;
