//! Domain services implementing cache-first patterns over source APIs.

mod music;
mod podcast;

pub use music::MusicService;
pub use podcast::PodcastService;
