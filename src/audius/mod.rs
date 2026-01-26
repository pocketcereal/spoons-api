//! Audius API client module.
//!
//! Provides integration with the Audius decentralized music streaming platform.
//! See: https://audiusproject.github.io/api-docs/

mod client;
mod types;

pub use client::AudiusClient;
pub use types::{AudiusTrack, AudiusUser};
