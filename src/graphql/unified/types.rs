use async_graphql::{Enum, SimpleObject};

use crate::graphql::audiobook::Audiobook;
use crate::graphql::podcast::{Episode, Podcast};
use crate::graphql::types::{Artist, Track};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum)]
pub enum ContentDomain {
    Music,
    Podcasts,
    Audiobooks,
}

// ==================== Search Results ====================

#[derive(Default, SimpleObject)]
pub struct SearchResults {
    pub music: Option<MusicSearchResults>,
    pub podcasts: Option<PodcastSearchResults>,
    pub audiobooks: Option<AudiobookSearchResults>,
}

#[derive(SimpleObject)]
pub struct MusicSearchResults {
    pub artists: Vec<Artist>,
    pub tracks: Vec<Track>,
}

#[derive(SimpleObject)]
pub struct PodcastSearchResults {
    pub podcasts: Vec<Podcast>,
}

#[derive(SimpleObject)]
pub struct AudiobookSearchResults {
    pub audiobooks: Vec<Audiobook>,
}

// ==================== Random Results ====================

#[derive(Default, SimpleObject)]
pub struct RandomResults {
    pub music: Option<MusicRandomResults>,
    pub podcasts: Option<PodcastRandomResults>,
    pub audiobooks: Option<AudiobookRandomResults>,
}

#[derive(SimpleObject)]
pub struct MusicRandomResults {
    pub artists: Vec<Artist>,
    pub tracks: Vec<Track>,
}

#[derive(SimpleObject)]
pub struct PodcastRandomResults {
    pub episodes: Vec<Episode>,
}

#[derive(SimpleObject)]
pub struct AudiobookRandomResults {
    pub audiobooks: Vec<Audiobook>,
}
