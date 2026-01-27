//! PodcastIndex API client and types.

mod auth;
mod client;
mod types;

pub use auth::{AuthHeaders, PodcastIndexAuth};
pub use client::PodcastIndexClient;
pub use types::{
    Category, PodcastEpisode, PodcastFeed, PodcastIndexEpisodesResponse,
    PodcastIndexPodcastResponse, PodcastIndexResponse,
};
