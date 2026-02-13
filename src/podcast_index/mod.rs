//! PodcastIndex API client and types.

mod auth;
mod client;
pub(crate) mod conversions;
pub(crate) mod endpoints;
pub(crate) mod types;

pub use client::PodcastIndexClient;
