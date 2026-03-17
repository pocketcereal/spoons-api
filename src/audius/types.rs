//! Based on the Audius API documentation:
//! https://audiusproject.github.io/api-docs/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudiusResponse<T> {
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudiusUser {
    pub id: String,
    pub handle: String,
    pub name: String,
    pub bio: Option<String>,
    pub location: Option<String>,
    #[serde(default)]
    pub is_verified: bool,
    #[serde(default)]
    pub is_deactivated: bool,
    #[serde(default)]
    pub follower_count: i64,
    #[serde(default)]
    pub followee_count: i64,
    #[serde(default)]
    pub track_count: i64,
    #[serde(default)]
    pub playlist_count: i64,
    #[serde(default)]
    pub repost_count: i64,
    pub profile_picture: Option<MultiSizeImage>,
    pub cover_photo: Option<CoverPhoto>,
    pub erc_wallet: Option<String>,
    pub spl_wallet: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverPhoto {
    #[serde(rename = "640x")]
    pub small: Option<String>,
    #[serde(rename = "2000x")]
    pub large: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudiusTrack {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub genre: Option<String>,
    pub mood: Option<String>,
    pub tags: Option<String>,
    #[serde(default)]
    pub duration: i64,
    #[serde(default)]
    pub play_count: i64,
    #[serde(default)]
    pub favorite_count: i64,
    #[serde(default)]
    pub repost_count: i64,
    pub release_date: Option<String>,
    #[serde(default)]
    pub is_streamable: bool,
    pub artwork: Option<MultiSizeImage>,
    pub permalink: Option<String>,
    pub user: Option<AudiusUser>,
}

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
