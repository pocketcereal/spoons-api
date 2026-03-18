use serde::Deserialize;

// `headers` is part of the Jamendo API contract; the provider will use it for error inspection.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct JamendoResponse<T> {
    pub headers: JamendoHeaders,
    pub results: Vec<T>,
}

// Fields are part of the public API response shape; the provider will use them.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct JamendoHeaders {
    pub status: String,
    pub code: i32,
    pub results_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JamendoTrack {
    pub id: String,
    pub name: String,
    pub duration: i32,
    pub artist_id: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub album_id: Option<String>,
    pub audio: String,
    pub audiodownload: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JamendoArtist {
    pub id: String,
    pub name: String,
    pub website: Option<String>,
    pub image: Option<String>,
    pub joindate: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_jamendo_track() {
        let json = r#"{
            "id": "123",
            "name": "Test Track",
            "duration": 180,
            "artist_id": "456",
            "artist_name": "Test Artist",
            "album_name": "Test Album",
            "album_id": "789",
            "audio": "https://storage.jamendo.com/tracks/stream/123",
            "audiodownload": "https://storage.jamendo.com/tracks/download/123",
            "image": "https://imgjam.com/artists/456/covers/1.200.jpg"
        }"#;

        let track: JamendoTrack = serde_json::from_str(json).unwrap();
        assert_eq!(track.id, "123");
        assert_eq!(track.name, "Test Track");
        assert_eq!(track.duration, 180);
        assert_eq!(track.artist_id, "456");
        assert_eq!(track.artist_name, "Test Artist");
        assert_eq!(track.album_name, Some("Test Album".to_string()));
        assert_eq!(track.audio, "https://storage.jamendo.com/tracks/stream/123");
    }

    #[test]
    fn test_deserialize_jamendo_track_minimal() {
        let json = r#"{
            "id": "999",
            "name": "Minimal Track",
            "duration": 60,
            "artist_id": "111",
            "artist_name": "Minimal Artist",
            "audio": "https://storage.jamendo.com/tracks/stream/999"
        }"#;

        let track: JamendoTrack = serde_json::from_str(json).unwrap();
        assert_eq!(track.id, "999");
        assert_eq!(track.album_name, None);
        assert_eq!(track.image, None);
        assert_eq!(track.audiodownload, None);
    }

    #[test]
    fn test_deserialize_jamendo_artist() {
        let json = r#"{
            "id": "456",
            "name": "Test Artist",
            "website": "https://example.com",
            "image": "https://imgjam.com/artists/456/covers/1.200.jpg",
            "joindate": "2010-01-01"
        }"#;

        let artist: JamendoArtist = serde_json::from_str(json).unwrap();
        assert_eq!(artist.id, "456");
        assert_eq!(artist.name, "Test Artist");
        assert_eq!(artist.website, Some("https://example.com".to_string()));
        assert_eq!(artist.joindate, Some("2010-01-01".to_string()));
    }

    #[test]
    fn test_deserialize_jamendo_artist_minimal() {
        let json = r#"{
            "id": "789",
            "name": "Minimal Artist"
        }"#;

        let artist: JamendoArtist = serde_json::from_str(json).unwrap();
        assert_eq!(artist.id, "789");
        assert_eq!(artist.website, None);
        assert_eq!(artist.image, None);
        assert_eq!(artist.joindate, None);
    }

    #[test]
    fn test_deserialize_jamendo_response() {
        let json = r#"{
            "headers": {
                "status": "success",
                "code": 0,
                "results_count": 1
            },
            "results": [
                {
                    "id": "123",
                    "name": "Test Track",
                    "duration": 180,
                    "artist_id": "456",
                    "artist_name": "Test Artist",
                    "audio": "https://storage.jamendo.com/tracks/stream/123"
                }
            ]
        }"#;

        let response: JamendoResponse<JamendoTrack> = serde_json::from_str(json).unwrap();
        assert_eq!(response.headers.status, "success");
        assert_eq!(response.headers.code, 0);
        assert_eq!(response.headers.results_count, 1);
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].id, "123");
    }
}
