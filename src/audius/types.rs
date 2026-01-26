//! Audius API response types.
//!
//! Based on the Audius API documentation:
//! https://audiusproject.github.io/api-docs/

use serde::{Deserialize, Serialize};

/// Wrapper for Audius API responses.
/// All Audius responses wrap the actual data in a `data` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudiusResponse<T> {
    pub data: T,
}

/// User (artist) entity from Audius.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudiusUser {
    /// Audius user ID.
    pub id: String,
    /// User handle (username).
    pub handle: String,
    /// Display name.
    pub name: String,
    /// User bio/description.
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
    pub followee_count: i64,
    /// Number of tracks uploaded.
    pub track_count: i64,
    /// Number of playlists created.
    pub playlist_count: i64,
    /// Number of reposts.
    pub repost_count: i64,
    /// Profile picture URLs.
    pub profile_picture: Option<MultiSizeImage>,
    /// Cover photo URLs.
    pub cover_photo: Option<CoverPhoto>,
    /// Ethereum wallet address.
    pub erc_wallet: Option<String>,
    /// Solana wallet address.
    pub spl_wallet: Option<String>,
    /// Total AUDIO token balance.
    pub total_audio_balance: Option<i64>,
}

/// Image with multiple sizes (150x150, 480x480, 1000x1000).
/// Used for profile pictures and track artwork.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSizeImage {
    #[serde(rename = "150x150")]
    pub small: Option<String>,
    #[serde(rename = "480x480")]
    pub medium: Option<String>,
    #[serde(rename = "1000x1000")]
    pub large: Option<String>,
}

/// Cover photo with multiple sizes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverPhoto {
    #[serde(rename = "640x")]
    pub small: Option<String>,
    #[serde(rename = "2000x")]
    pub large: Option<String>,
}

/// Track entity from Audius.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudiusTrack {
    /// Audius track ID.
    pub id: String,
    /// Track title.
    pub title: String,
    /// Track description.
    pub description: Option<String>,
    /// Genre classification.
    pub genre: Option<String>,
    /// Mood classification.
    pub mood: Option<String>,
    /// Track tags.
    pub tags: Option<String>,
    /// Duration in seconds.
    pub duration: i64,
    /// Number of plays.
    pub play_count: i64,
    /// Number of favorites.
    pub favorite_count: i64,
    /// Number of reposts.
    pub repost_count: i64,
    /// Release date.
    pub release_date: Option<String>,
    /// Whether the track is streamable.
    pub is_streamable: bool,
    /// Artwork URLs.
    pub artwork: Option<MultiSizeImage>,
    /// Track permalink (URL path).
    pub permalink: Option<String>,
    /// The user who uploaded the track.
    pub user: Option<AudiusUser>,
}

/// Host discovery response from api.audius.co.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostDiscoveryResponse {
    pub data: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_deserialize() {
        let json = r#"{
            "id": "abc123",
            "handle": "testuser",
            "name": "Test User",
            "bio": "A test user",
            "is_verified": true,
            "is_deactivated": false,
            "follower_count": 100,
            "followee_count": 50,
            "track_count": 10,
            "playlist_count": 5,
            "repost_count": 20
        }"#;
        let user: AudiusUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.handle, "testuser");
        assert_eq!(user.follower_count, 100);
        assert!(user.is_verified);
    }

    #[test]
    fn test_track_deserialize() {
        let json = r#"{
            "id": "track123",
            "title": "Test Track",
            "genre": "Electronic",
            "duration": 180,
            "play_count": 1000,
            "favorite_count": 50,
            "repost_count": 10,
            "is_streamable": true
        }"#;
        let track: AudiusTrack = serde_json::from_str(json).unwrap();
        assert_eq!(track.title, "Test Track");
        assert_eq!(track.duration, 180);
        assert!(track.is_streamable);
    }

    #[test]
    fn test_response_wrapper() {
        let json = r#"{
            "data": [
                {
                    "id": "abc123",
                    "handle": "user1",
                    "name": "User One",
                    "is_verified": false,
                    "is_deactivated": false,
                    "follower_count": 10,
                    "followee_count": 5,
                    "track_count": 2,
                    "playlist_count": 1,
                    "repost_count": 0
                }
            ]
        }"#;
        let response: AudiusResponse<Vec<AudiusUser>> = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].handle, "user1");
    }
}
