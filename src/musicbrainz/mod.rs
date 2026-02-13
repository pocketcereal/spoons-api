//! MusicBrainz API client and types.

mod client;
mod types;

pub use client::MusicBrainzClient;
pub use types::{
    Area, Artist, ArtistCredit, ArtistCreditArtist, LifeSpan, Recording, Release, ReleaseGroup,
};
