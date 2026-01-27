//! GraphQL podcast types with source-agnostic interfaces.

mod episode_types;
mod podcast_types;
mod queries;

pub use episode_types::{Episode, PodcastIndexEpisode};
pub use podcast_types::{Category, Podcast, PodcastIndexPodcast, PodcastSource};
pub use queries::PodcastQuery;
