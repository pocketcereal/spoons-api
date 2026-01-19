//! Test database setup and teardown utilities.

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

/// Type alias for the test database pool.
pub type TestPool = Pool<AsyncPgConnection>;

/// Test database helper for setup and teardown.
pub struct TestDb {
    pub pool: TestPool,
}

impl TestDb {
    /// Create a new test database connection pool.
    pub async fn new() -> Self {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://spoons:spoons@localhost:5432/spoons_test".to_string());

        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
        let pool = Pool::builder(manager)
            .max_size(2)
            .build()
            .expect("Failed to create test database pool");

        Self { pool }
    }

    /// Truncate all cache tables before tests.
    pub async fn truncate_tables(&self) {
        let mut conn = self.pool.get().await.expect("Failed to get connection");

        // Truncate all tables in a single statement to avoid deadlocks
        diesel::sql_query(
            "TRUNCATE TABLE
                artist_search_cache,
                release_search_cache,
                recording_search_cache,
                release_group_search_cache,
                releases,
                recordings,
                release_groups,
                artists,
                areas
            CASCADE"
        )
        .execute(&mut conn)
        .await
        .expect("Failed to truncate tables");
    }
}
