//! HTTP client module with factory pattern for API clients.

use std::time::Duration;

mod client;
mod factory;

pub use client::ApiClient;
pub use factory::ClientConfig;

/// Default timeout for API requests (30 seconds).
pub const DEFAULT_API_TIMEOUT: Duration = Duration::from_secs(30);

/// Timeout for host discovery requests (10 seconds).
pub const HOST_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10);
