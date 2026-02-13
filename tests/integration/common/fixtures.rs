//! Test fixtures for integration tests.
//!
//! These fixtures use real MusicBrainz UUIDs for well-known artists, releases, etc.

#![allow(dead_code)]
#![allow(unused_imports)]

use spoons_api::musicbrainz::{Area, Artist, LifeSpan, Recording, Release, ReleaseGroup};
use spoons_api::podcast::{Category, Episode, Podcast};

/// Nirvana artist fixture (real MusicBrainz UUID).
pub fn nirvana_artist() -> Artist {
    Artist {
        id: "5b11f4ce-a62d-471e-81fc-a69a8278c7da".to_string(),
        name: "Nirvana".to_string(),
        sort_name: Some("Nirvana".to_string()),
        artist_type: Some("Group".to_string()),
        country: Some("US".to_string()),
        area: Some(Area {
            id: "489ce91b-6658-3307-9877-795b68554c98".to_string(),
            name: "United States".to_string(),
            sort_name: Some("United States".to_string()),
        }),
        disambiguation: Some("90s grunge band".to_string()),
        life_span: Some(LifeSpan {
            begin: Some("1987".to_string()),
            end: Some("1994".to_string()),
            ended: Some(true),
        }),
    }
}

/// Radiohead artist fixture (real MusicBrainz UUID).
pub fn radiohead_artist() -> Artist {
    Artist {
        id: "a74b1b7f-71a5-4011-9441-d0b5e4122711".to_string(),
        name: "Radiohead".to_string(),
        sort_name: Some("Radiohead".to_string()),
        artist_type: Some("Group".to_string()),
        country: Some("GB".to_string()),
        area: Some(Area {
            id: "8a754a16-0027-3a29-b6d7-2b40ea0481ed".to_string(),
            name: "United Kingdom".to_string(),
            sort_name: Some("United Kingdom".to_string()),
        }),
        disambiguation: None,
        life_span: Some(LifeSpan {
            begin: Some("1985".to_string()),
            end: None,
            ended: Some(false),
        }),
    }
}

/// Nevermind release group fixture (real MusicBrainz UUID).
pub fn nevermind_release_group() -> ReleaseGroup {
    ReleaseGroup {
        id: "1b022e01-4da6-387b-8658-8678046e4cef".to_string(),
        title: "Nevermind".to_string(),
        primary_type: Some("Album".to_string()),
        secondary_types: None,
        first_release_date: Some("1991-09-24".to_string()),
        disambiguation: None,
    }
}

/// OK Computer release group fixture (real MusicBrainz UUID).
pub fn ok_computer_release_group() -> ReleaseGroup {
    ReleaseGroup {
        id: "6108f66e-6e75-34d0-a64f-77340185c2ac".to_string(),
        title: "OK Computer".to_string(),
        primary_type: Some("Album".to_string()),
        secondary_types: None,
        first_release_date: Some("1997-05-21".to_string()),
        disambiguation: None,
    }
}

/// Nevermind release fixture (real MusicBrainz UUID).
pub fn nevermind_release() -> Release {
    Release {
        id: "b52a8f31-b5ab-34e9-92f4-f5b7110f8c3f".to_string(),
        title: "Nevermind".to_string(),
        status: Some("Official".to_string()),
        date: Some("1991-09-24".to_string()),
        country: Some("US".to_string()),
        barcode: Some("720642442524".to_string()),
        disambiguation: None,
        release_group: Some(nevermind_release_group()),
    }
}

/// OK Computer release fixture (real MusicBrainz UUID).
pub fn ok_computer_release() -> Release {
    Release {
        id: "382f1005-e9ab-4684-afd4-0bdae4ee37f2".to_string(),
        title: "OK Computer".to_string(),
        status: Some("Official".to_string()),
        date: Some("1997-06-16".to_string()),
        country: Some("US".to_string()),
        barcode: Some("724385522925".to_string()),
        disambiguation: None,
        release_group: Some(ok_computer_release_group()),
    }
}

/// Smells Like Teen Spirit recording fixture (real MusicBrainz UUID).
pub fn smells_like_teen_spirit_recording() -> Recording {
    Recording {
        id: "f44f4f7c-8a05-4e0c-892a-fc1c6e9fb9d2".to_string(),
        title: "Smells Like Teen Spirit".to_string(),
        length: Some(301000), // ~5:01
        disambiguation: None,
        video: Some(false),
        artist_credit: Vec::new(),
    }
}

/// Paranoid Android recording fixture (real MusicBrainz UUID).
pub fn paranoid_android_recording() -> Recording {
    Recording {
        id: "9f9cf187-d6f9-437f-9d98-d59cdbd52757".to_string(),
        title: "Paranoid Android".to_string(),
        length: Some(383000), // ~6:23
        disambiguation: None,
        video: Some(false),
        artist_credit: Vec::new(),
    }
}

/// Come as You Are recording fixture.
pub fn come_as_you_are_recording() -> Recording {
    Recording {
        id: "9b8b0e1a-c871-4405-9b44-f97b25243c4c".to_string(),
        title: "Come as You Are".to_string(),
        length: Some(219000), // ~3:39
        disambiguation: None,
        video: Some(false),
        artist_credit: Vec::new(),
    }
}

/// Karma Police recording fixture.
pub fn karma_police_recording() -> Recording {
    Recording {
        id: "a776b426-26ee-4dd7-9cc3-8c2ea7a6bdfe".to_string(),
        title: "Karma Police".to_string(),
        length: Some(264000), // ~4:24
        disambiguation: None,
        video: Some(false),
        artist_credit: Vec::new(),
    }
}

// ==================== Podcast Fixtures ====================

/// Syntax podcast fixture (real PodcastIndex ID).
pub fn syntax_podcast() -> Podcast {
    Podcast {
        id: 920666, // Real PodcastIndex ID for "Syntax - Tasty Web Development Treats"
        title: "Syntax - Tasty Web Development Treats".to_string(),
        author: Some("Wes Bos & Scott Tolinski".to_string()),
        description: Some("A podcast about web development".to_string()),
        artwork_url: Some("https://example.com/syntax-artwork.jpg".to_string()),
        feed_url: "https://feed.syntax.fm/rss".to_string(),
        language: Some("en".to_string()),
        categories: vec![Category {
            id: 102,
            name: "Technology".to_string(),
        }],
        episode_count: Some(500),
        latest_publish_time: None,
        itunes_id: Some(1253186678),
        trend_score: Some(42),
        podcast_guid: Some("abc-123-syntax".to_string()),
    }
}

/// The Daily podcast fixture.
pub fn the_daily_podcast() -> Podcast {
    Podcast {
        id: 1200361, // Example ID
        title: "The Daily".to_string(),
        author: Some("The New York Times".to_string()),
        description: Some(
            "This is what the news should sound like. The biggest stories of our time.".to_string(),
        ),
        artwork_url: Some("https://example.com/daily-artwork.jpg".to_string()),
        feed_url: "https://feeds.simplecast.com/54nAGcIl".to_string(),
        language: Some("en".to_string()),
        categories: vec![
            Category {
                id: 99,
                name: "News".to_string(),
            },
            Category {
                id: 100,
                name: "Politics".to_string(),
            },
        ],
        episode_count: Some(1200),
        latest_publish_time: None,
        itunes_id: Some(1200361),
        trend_score: Some(95),
        podcast_guid: Some("def-456-daily".to_string()),
    }
}

/// Syntax episode fixture - JavaScript Performance Tips.
pub fn syntax_episode_1() -> Episode {
    Episode {
        id: 12345678,
        podcast_id: 920666, // Matches syntax_podcast()
        title: "JavaScript Performance Tips".to_string(),
        description: Some("Tips for making your JS faster".to_string()),
        audio_url: "https://example.com/syntax-ep123.mp3".to_string(),
        audio_type: Some("audio/mpeg".to_string()),
        audio_length: Some(45000000), // ~45MB
        duration_seconds: Some(3600), // 1 hour
        published_at: None,
        episode_number: Some(123),
        season_number: Some(1),
        episode_type: Some("full".to_string()),
        image_url: None,
        explicit: Some(false),
    }
}

/// Syntax episode fixture - TypeScript Deep Dive.
pub fn syntax_episode_2() -> Episode {
    Episode {
        id: 12345679,
        podcast_id: 920666, // Matches syntax_podcast()
        title: "TypeScript Deep Dive".to_string(),
        description: Some("Everything you need to know about TypeScript".to_string()),
        audio_url: "https://example.com/syntax-ep124.mp3".to_string(),
        audio_type: Some("audio/mpeg".to_string()),
        audio_length: Some(42000000),
        duration_seconds: Some(3300),
        published_at: None,
        episode_number: Some(124),
        season_number: Some(1),
        episode_type: Some("full".to_string()),
        image_url: Some("https://example.com/syntax-ep124.jpg".to_string()),
        explicit: Some(false),
    }
}

/// The Daily episode fixture.
pub fn daily_episode() -> Episode {
    Episode {
        id: 87654321,
        podcast_id: 1200361, // Matches the_daily_podcast()
        title: "The Future of Climate Policy".to_string(),
        description: Some("An in-depth look at global climate agreements".to_string()),
        audio_url: "https://example.com/daily-ep456.mp3".to_string(),
        audio_type: Some("audio/mpeg".to_string()),
        audio_length: Some(30000000),
        duration_seconds: Some(1800), // 30 minutes
        published_at: None,
        episode_number: Some(456),
        season_number: None,
        episode_type: Some("full".to_string()),
        image_url: None,
        explicit: Some(false),
    }
}
