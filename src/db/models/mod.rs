//! Database models for music cache entities.

mod area;
mod artist;
mod audiobook;
mod chapter;
mod episode;
mod podcast;
mod recording;
mod release;
mod release_group;
mod search_cache;

pub use area::{AreaRow, NewAreaRow};
pub use artist::{ArtistRow, NewArtistRow};
pub use audiobook::{AudiobookRow, NewAudiobookRow};
pub use chapter::{ChapterRow, NewChapterRow};
pub use episode::{EpisodeRow, NewEpisodeRow};
pub use podcast::{NewPodcastRow, PodcastRow};
pub use recording::{NewRecordingRow, RecordingRow};
pub use release::{NewReleaseRow, ReleaseRow};
pub use release_group::{NewReleaseGroupRow, ReleaseGroupRow};
pub use search_cache::{
    ArtistSearchCacheRow, AudiobookSearchCacheRow, NewArtistSearchCacheRow,
    NewAudiobookSearchCacheRow, NewPodcastSearchCacheRow, NewRecordingSearchCacheRow,
    NewReleaseGroupSearchCacheRow, NewReleaseSearchCacheRow, PodcastSearchCacheRow,
    RecordingSearchCacheRow, ReleaseGroupSearchCacheRow, ReleaseSearchCacheRow,
};
