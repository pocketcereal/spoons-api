//! MusicBrainz API client and types.

mod client;
mod types;

#[allow(unused_imports)]
pub use client::MusicBrainzClient;
pub use types::{Area, Artist, LifeSpan, Recording, Release, ReleaseGroup};
