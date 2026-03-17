//! GraphQL types with source-agnostic interfaces.
//!
//! This module defines GraphQL interfaces that abstract over multiple data sources,
//! allowing mixed search results from MusicBrainz, Audius, and future sources.
//!
//! Note: The Artist and Track interface field implementations are intentionally
//! duplicated across concrete types. async-graphql's `#[Object]` proc macro
//! doesn't support `macro_rules!` expansion inside impl blocks.

use async_graphql::{Interface, Object, SimpleObject};

use crate::domain::DataSource;

// ============================================================================
// Artist Interface and Implementations
// ============================================================================

/// Artist interface - common fields across all music data sources.
///
/// Concrete implementations:
/// - `MusicBrainzArtist` - Artist from MusicBrainz
/// - `AudiusArtist` - User/Artist from Audius
#[allow(clippy::duplicated_attributes)] // False positive: each field() is a distinct interface field
#[derive(Interface)]
#[graphql(
    field(name = "id", ty = "String", desc = "Unique identifier (internal)"),
    field(name = "name", ty = "String", desc = "Artist/user display name"),
    field(
        name = "source",
        ty = "DataSource",
        desc = "Data source this record came from"
    ),
    field(name = "source_id", ty = "String", desc = "ID in the source system"),
    field(
        name = "image_url",
        ty = "Option<String>",
        desc = "Profile/artist image URL"
    )
)]
pub enum Artist {
    MusicBrainz(MusicBrainzArtist),
    Audius(AudiusArtist),
}

/// Artist from MusicBrainz.
#[derive(Debug, Clone)]
pub struct MusicBrainzArtist {
    /// Internal ID (prefixed with source).
    pub id: String,
    /// Artist name.
    pub name: String,
    /// MusicBrainz ID.
    pub source_id: String,
    /// Image URL.
    pub image_url: Option<String>,
    /// Sort name for the artist.
    pub sort_name: Option<String>,
    /// Artist type (person, group, etc.).
    pub artist_type: Option<String>,
    /// Disambiguation comment.
    pub disambiguation: Option<String>,
    /// Country of origin.
    pub country: Option<String>,
    /// Area of origin.
    pub area: Option<MusicBrainzArea>,
    /// Life span information.
    pub life_span: Option<MusicBrainzLifeSpan>,
}

#[Object]
impl MusicBrainzArtist {
    /// Internal ID.
    async fn id(&self) -> &str {
        &self.id
    }

    /// Artist name.
    async fn name(&self) -> &str {
        &self.name
    }

    /// Data source.
    async fn source(&self) -> DataSource {
        DataSource::MusicBrainz
    }

    /// MusicBrainz ID.
    async fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Image URL.
    async fn image_url(&self) -> Option<String> {
        self.image_url.clone()
    }

    // MusicBrainz-specific fields

    /// Sort name for the artist.
    async fn sort_name(&self) -> Option<&str> {
        self.sort_name.as_deref()
    }

    /// Artist type (person, group, etc.).
    async fn artist_type(&self) -> Option<&str> {
        self.artist_type.as_deref()
    }

    /// Disambiguation comment.
    async fn disambiguation(&self) -> Option<&str> {
        self.disambiguation.as_deref()
    }

    /// Country of origin.
    async fn country(&self) -> Option<&str> {
        self.country.as_deref()
    }

    /// Area of origin.
    async fn area(&self) -> Option<&MusicBrainzArea> {
        self.area.as_ref()
    }

    /// Life span information.
    async fn life_span(&self) -> Option<&MusicBrainzLifeSpan> {
        self.life_span.as_ref()
    }
}

/// Area from MusicBrainz.
#[derive(Debug, Clone, SimpleObject)]
pub struct MusicBrainzArea {
    /// MusicBrainz ID.
    pub id: String,
    /// Area name.
    pub name: String,
    /// Sort name.
    pub sort_name: Option<String>,
}

/// Life span from MusicBrainz.
#[derive(Debug, Clone, SimpleObject)]
pub struct MusicBrainzLifeSpan {
    /// Begin date.
    pub begin: Option<String>,
    /// End date.
    pub end: Option<String>,
    /// Whether the entity has ended.
    pub ended: Option<bool>,
}

/// Artist/User from Audius.
#[derive(Debug, Clone)]
pub struct AudiusArtist {
    /// Internal ID (prefixed with source).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Audius user ID.
    pub source_id: String,
    /// Profile picture URL.
    pub image_url: Option<String>,
    /// User handle (username).
    pub handle: String,
    /// User bio.
    pub bio: Option<String>,
    /// User location.
    pub location: Option<String>,
    /// Whether the user is verified.
    pub is_verified: bool,
    /// Whether the user is deactivated.
    pub is_deactivated: bool,
    /// Number of followers.
    pub follower_count: i64,
    /// Number of users being followed.
    pub following_count: i64,
    /// Number of tracks.
    pub track_count: i64,
    /// Number of playlists.
    pub playlist_count: i64,
}

#[Object]
impl AudiusArtist {
    /// Internal ID.
    async fn id(&self) -> &str {
        &self.id
    }

    /// Display name.
    async fn name(&self) -> &str {
        &self.name
    }

    /// Data source.
    async fn source(&self) -> DataSource {
        DataSource::Audius
    }

    /// Audius user ID.
    async fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Profile picture URL.
    async fn image_url(&self) -> Option<String> {
        self.image_url.clone()
    }

    // Audius-specific fields

    /// User handle (username).
    async fn handle(&self) -> &str {
        &self.handle
    }

    /// User bio.
    async fn bio(&self) -> Option<&str> {
        self.bio.as_deref()
    }

    /// User location.
    async fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    /// Whether the user is verified.
    async fn is_verified(&self) -> bool {
        self.is_verified
    }

    /// Whether the user is deactivated.
    async fn is_deactivated(&self) -> bool {
        self.is_deactivated
    }

    /// Number of followers.
    async fn follower_count(&self) -> i64 {
        self.follower_count
    }

    /// Number of users being followed.
    async fn following_count(&self) -> i64 {
        self.following_count
    }

    /// Number of tracks.
    async fn track_count(&self) -> i64 {
        self.track_count
    }

    /// Number of playlists.
    async fn playlist_count(&self) -> i64 {
        self.playlist_count
    }
}

// ============================================================================
// Track Interface and Implementations
// ============================================================================

/// Track interface - common fields across all music data sources.
#[allow(clippy::duplicated_attributes)] // False positive: each field() is a distinct interface field
#[derive(Interface)]
#[graphql(
    field(name = "id", ty = "String", desc = "Unique identifier (internal)"),
    field(name = "title", ty = "String", desc = "Track title"),
    field(
        name = "source",
        ty = "DataSource",
        desc = "Data source this record came from"
    ),
    field(name = "source_id", ty = "String", desc = "ID in the source system"),
    field(
        name = "duration_ms",
        ty = "Option<i64>",
        desc = "Duration in milliseconds"
    ),
    field(
        name = "artist_name",
        ty = "Option<String>",
        desc = "Primary artist name"
    )
)]
pub enum Track {
    MusicBrainz(MusicBrainzTrack),
    Audius(AudiusTrack),
}

/// Recording/Track from MusicBrainz.
#[derive(Debug, Clone)]
pub struct MusicBrainzTrack {
    /// Internal ID.
    pub id: String,
    /// Track title.
    pub title: String,
    /// MusicBrainz recording ID.
    pub source_id: String,
    /// Duration in milliseconds.
    pub duration_ms: Option<i64>,
    /// Primary artist name.
    pub artist_name: Option<String>,
    /// Disambiguation comment.
    pub disambiguation: Option<String>,
    /// Whether this is a video recording.
    pub video: Option<bool>,
}

#[Object]
impl MusicBrainzTrack {
    /// Internal ID.
    async fn id(&self) -> &str {
        &self.id
    }

    /// Track title.
    async fn title(&self) -> &str {
        &self.title
    }

    /// Data source.
    async fn source(&self) -> DataSource {
        DataSource::MusicBrainz
    }

    /// MusicBrainz recording ID.
    async fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Duration in milliseconds.
    async fn duration_ms(&self) -> Option<i64> {
        self.duration_ms
    }

    /// Primary artist name.
    async fn artist_name(&self) -> Option<String> {
        self.artist_name.clone()
    }

    // MusicBrainz-specific fields

    /// Disambiguation comment.
    async fn disambiguation(&self) -> Option<&str> {
        self.disambiguation.as_deref()
    }

    /// Whether this is a video recording.
    async fn video(&self) -> Option<bool> {
        self.video
    }
}

/// Track from Audius.
#[derive(Debug, Clone)]
pub struct AudiusTrack {
    /// Internal ID.
    pub id: String,
    /// Track title.
    pub title: String,
    /// Audius track ID.
    pub source_id: String,
    /// Duration in milliseconds.
    pub duration_ms: Option<i64>,
    /// Primary artist name.
    pub artist_name: Option<String>,
    /// Track description.
    pub description: Option<String>,
    /// Genre.
    pub genre: Option<String>,
    /// Mood.
    pub mood: Option<String>,
    /// Number of plays.
    pub play_count: i64,
    /// Number of favorites.
    pub favorite_count: i64,
    /// Number of reposts.
    pub repost_count: i64,
    /// Artwork URL.
    pub artwork_url: Option<String>,
    /// Whether the track is streamable.
    pub is_streamable: bool,
}

#[Object]
impl AudiusTrack {
    /// Internal ID.
    async fn id(&self) -> &str {
        &self.id
    }

    /// Track title.
    async fn title(&self) -> &str {
        &self.title
    }

    /// Data source.
    async fn source(&self) -> DataSource {
        DataSource::Audius
    }

    /// Audius track ID.
    async fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Duration in milliseconds.
    async fn duration_ms(&self) -> Option<i64> {
        self.duration_ms
    }

    /// Primary artist name.
    async fn artist_name(&self) -> Option<String> {
        self.artist_name.clone()
    }

    // Audius-specific fields

    /// Track description.
    async fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Genre.
    async fn genre(&self) -> Option<&str> {
        self.genre.as_deref()
    }

    /// Mood.
    async fn mood(&self) -> Option<&str> {
        self.mood.as_deref()
    }

    /// Number of plays.
    async fn play_count(&self) -> i64 {
        self.play_count
    }

    /// Number of favorites.
    async fn favorite_count(&self) -> i64 {
        self.favorite_count
    }

    /// Number of reposts.
    async fn repost_count(&self) -> i64 {
        self.repost_count
    }

    /// Artwork URL.
    async fn artwork_url(&self) -> Option<&str> {
        self.artwork_url.as_deref()
    }

    /// Whether the track is streamable.
    async fn is_streamable(&self) -> bool {
        self.is_streamable
    }
}

// ============================================================================
// Conversion Implementations
// ============================================================================

impl From<crate::musicbrainz::Artist> for MusicBrainzArtist {
    fn from(artist: crate::musicbrainz::Artist) -> Self {
        let prefixed_id = DataSource::MusicBrainz.format_id(&artist.id);
        Self {
            id: prefixed_id,
            source_id: artist.id,
            name: artist.name,
            image_url: None, // MusicBrainz doesn't include images in artist response
            sort_name: artist.sort_name,
            artist_type: artist.artist_type,
            disambiguation: artist.disambiguation,
            country: artist.country,
            area: artist.area.map(|a| MusicBrainzArea {
                id: a.id,
                name: a.name,
                sort_name: a.sort_name,
            }),
            life_span: artist.life_span.map(|ls| MusicBrainzLifeSpan {
                begin: ls.begin,
                end: ls.end,
                ended: ls.ended,
            }),
        }
    }
}

impl From<crate::musicbrainz::Recording> for MusicBrainzTrack {
    fn from(recording: crate::musicbrainz::Recording) -> Self {
        let prefixed_id = DataSource::MusicBrainz.format_id(&recording.id);
        let artist_name = format_artist_credits(&recording.artist_credit);
        Self {
            id: prefixed_id,
            source_id: recording.id,
            title: recording.title,
            duration_ms: recording.length,
            artist_name,
            disambiguation: recording.disambiguation,
            video: recording.video,
        }
    }
}

/// Formats artist credits into a combined artist name string.
/// e.g., "Artist A feat. Artist B & Artist C"
fn format_artist_credits(credits: &[crate::musicbrainz::ArtistCredit]) -> Option<String> {
    if credits.is_empty() {
        return None;
    }
    let mut result = String::new();
    for credit in credits {
        result.push_str(&credit.artist.name);
        if !credit.joinphrase.is_empty() {
            result.push_str(&credit.joinphrase);
        }
    }
    Some(result)
}

impl From<crate::audius::AudiusUser> for AudiusArtist {
    fn from(user: crate::audius::AudiusUser) -> Self {
        let prefixed_id = DataSource::Audius.format_id(&user.id);
        Self {
            id: prefixed_id,
            source_id: user.id,
            name: user.name,
            image_url: user.profile_picture.and_then(|p| p.medium.or(p.small)),
            handle: user.handle,
            bio: user.bio,
            location: user.location,
            is_verified: user.is_verified,
            is_deactivated: user.is_deactivated,
            follower_count: user.follower_count,
            following_count: user.followee_count,
            track_count: user.track_count,
            playlist_count: user.playlist_count,
        }
    }
}

impl From<crate::audius::AudiusTrack> for AudiusTrack {
    fn from(track: crate::audius::AudiusTrack) -> Self {
        let prefixed_id = DataSource::Audius.format_id(&track.id);
        Self {
            id: prefixed_id,
            source_id: track.id,
            title: track.title,
            duration_ms: Some(track.duration * 1000), // Convert seconds to ms
            artist_name: track.user.map(|u| u.name),
            description: track.description,
            genre: track.genre,
            mood: track.mood,
            play_count: track.play_count,
            favorite_count: track.favorite_count,
            repost_count: track.repost_count,
            artwork_url: track.artwork.and_then(|a| a.medium.or(a.small)),
            is_streamable: track.is_streamable,
        }
    }
}
