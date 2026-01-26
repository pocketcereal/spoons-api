//! Database models for music cache entities.

mod area;
mod artist;
mod recording;
mod release;
mod release_group;
mod search_cache;

pub use area::{AreaRow, NewAreaRow};
pub use artist::{ArtistRow, NewArtistRow};
pub use recording::{NewRecordingRow, RecordingRow};
pub use release::{NewReleaseRow, ReleaseRow};
pub use release_group::{NewReleaseGroupRow, ReleaseGroupRow};
pub use search_cache::{
    ArtistSearchCacheRow, NewArtistSearchCacheRow, NewRecordingSearchCacheRow,
    NewReleaseGroupSearchCacheRow, NewReleaseSearchCacheRow, RecordingSearchCacheRow,
    ReleaseGroupSearchCacheRow, ReleaseSearchCacheRow,
};
