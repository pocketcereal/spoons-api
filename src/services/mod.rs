//! Domain services implementing cache-first patterns over source APIs.

mod audiobook;
mod music;
mod podcast;

pub use audiobook::AudiobookService;
pub use music::MusicService;
pub use podcast::PodcastService;
