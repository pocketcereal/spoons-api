//! PodcastIndex API client and types.

mod auth;
mod client;
pub mod conversions;
pub mod endpoints;
mod types;

pub use auth::{AuthHeaders, PodcastIndexAuth};
pub use client::PodcastIndexClient;
pub use conversions::{episode_from_podcast_episode, podcast_from_feed};
pub use types::{
    Category, PodcastEpisode, PodcastFeed, PodcastIndexEpisodesResponse,
    PodcastIndexPodcastResponse, PodcastIndexResponse,
};
