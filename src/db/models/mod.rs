//! Database models for music cache entities.

mod area;
mod artist;
mod recording;
mod release;
mod release_group;
mod search_cache;

pub use area::{AreaRow, NewAreaRow};
pub use artist::{ArtistRow, NewArtistRow};
pub use recording::{RecordingRow, NewRecordingRow};
pub use release::{ReleaseRow, NewReleaseRow};
pub use release_group::{ReleaseGroupRow, NewReleaseGroupRow};
pub use search_cache::{
    ArtistSearchCacheRow, NewArtistSearchCacheRow,
    RecordingSearchCacheRow, NewRecordingSearchCacheRow,
    ReleaseGroupSearchCacheRow, NewReleaseGroupSearchCacheRow,
    ReleaseSearchCacheRow, NewReleaseSearchCacheRow,
};
