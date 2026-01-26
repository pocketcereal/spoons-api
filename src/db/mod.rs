//! Database module for PostgreSQL with diesel-async.

mod helpers;
pub mod models;
mod pool;
pub mod repositories;
pub mod schema;

pub use helpers::{
    MAX_BATCH_SIZE, db_error, get_conn, parse_uuid, spawn_cache_task, validate_batch_size,
};
pub use pool::{DbConfig, DbPool, create_pool};
pub use repositories::MusicRepository;
