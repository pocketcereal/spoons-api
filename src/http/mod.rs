//! HTTP client module with factory pattern for API clients.

mod client;
mod factory;

#[allow(unused_imports)]
pub use client::ApiClient;
#[allow(unused_imports)]
pub use factory::{ClientConfig, ClientFactory};
