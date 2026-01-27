//! Conversions from PodcastIndex API types to domain types.

use crate::podcast::{Category, Episode, Podcast};
use crate::podcast_index::types::{PodcastEpisode, PodcastFeed};
use chrono::{TimeZone, Utc};

/// Converts a PodcastIndex PodcastFeed to a domain Podcast.
pub fn podcast_from_feed(feed: PodcastFeed) -> Podcast {
    let categories = feed
        .categories
        .and_then(|cats| {
            if let serde_json::Value::Object(map) = cats {
                Some(
                    map.into_iter()
                        .filter_map(|(id_str, name_val)| {
                            let id = id_str.parse::<i32>().ok()?;
                            let name = name_val.as_str()?.to_string();
                            Some(Category { id, name })
                        })
                        .collect(),
                )
            } else {
                None
            }
        })
        .unwrap_or_default();

    let latest_publish_time = feed
        .newest_item_publish_time
        .and_then(|ts| Utc.timestamp_opt(ts, 0).single());

    Podcast {
        id: feed.id,
        title: feed.title,
        author: Some(feed.author),
        description: feed.description,
        artwork_url: feed.artwork.or(feed.image),
        feed_url: feed.url,
        language: Some(feed.language),
        categories,
        episode_count: None,
        latest_publish_time,
        itunes_id: feed.itunes_id,
        trend_score: feed.trend_score,
        podcast_guid: None,
    }
}

/// Converts a PodcastIndex PodcastEpisode to a domain Episode.
pub fn episode_from_podcast_episode(episode: PodcastEpisode) -> Episode {
    let published_at = Utc.timestamp_opt(episode.date_published, 0).single();

    Episode {
        id: episode.id,
        podcast_id: episode.feed_id,
        title: episode.title,
        description: episode.description,
        audio_url: episode.enclosure_url,
        audio_type: Some(episode.enclosure_type),
        audio_length: Some(episode.enclosure_length),
        duration_seconds: episode.duration,
        published_at,
        episode_number: episode.episode,
        season_number: episode.season,
        episode_type: episode.episode_type,
        image_url: episode.image,
        explicit: Some(episode.explicit != 0),
    }
}

impl From<PodcastFeed> for Podcast {
    fn from(feed: PodcastFeed) -> Self {
        podcast_from_feed(feed)
    }
}

impl From<PodcastEpisode> for Episode {
    fn from(episode: PodcastEpisode) -> Self {
        episode_from_podcast_episode(episode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_podcast_from_feed() {
        let feed = PodcastFeed {
            id: 6974466,
            title: "Test Podcast".to_string(),
            author: "Test Author".to_string(),
            description: Some("A test podcast".to_string()),
            image: Some("https://example.com/image.jpg".to_string()),
            artwork: Some("https://example.com/artwork.jpg".to_string()),
            url: "https://example.com/feed.xml".to_string(),
            itunes_id: Some(123456),
            language: "en".to_string(),
            categories: Some(serde_json::json!({"102": "Technology", "103": "News"})),
            newest_item_publish_time: Some(1634567890),
            trend_score: Some(42),
        };

        let podcast = podcast_from_feed(feed);

        assert_eq!(podcast.id, 6974466);
        assert_eq!(podcast.title, "Test Podcast");
        assert_eq!(podcast.author, Some("Test Author".to_string()));
        assert_eq!(
            podcast.artwork_url,
            Some("https://example.com/artwork.jpg".to_string())
        );
        assert_eq!(podcast.feed_url, "https://example.com/feed.xml");
        assert_eq!(podcast.language, Some("en".to_string()));
        assert_eq!(podcast.categories.len(), 2);
        assert_eq!(podcast.trend_score, Some(42));
        assert!(podcast.latest_publish_time.is_some());
    }

    #[test]
    fn test_podcast_from_feed_prefers_artwork() {
        let feed = PodcastFeed {
            id: 1,
            title: "Test".to_string(),
            author: "Author".to_string(),
            description: None,
            image: Some("https://example.com/image.jpg".to_string()),
            artwork: Some("https://example.com/artwork.jpg".to_string()),
            url: "https://example.com/feed.xml".to_string(),
            itunes_id: None,
            language: "en".to_string(),
            categories: None,
            newest_item_publish_time: None,
            trend_score: None,
        };

        let podcast = podcast_from_feed(feed);
        assert_eq!(
            podcast.artwork_url,
            Some("https://example.com/artwork.jpg".to_string())
        );
    }

    #[test]
    fn test_podcast_from_feed_falls_back_to_image() {
        let feed = PodcastFeed {
            id: 1,
            title: "Test".to_string(),
            author: "Author".to_string(),
            description: None,
            image: Some("https://example.com/image.jpg".to_string()),
            artwork: None,
            url: "https://example.com/feed.xml".to_string(),
            itunes_id: None,
            language: "en".to_string(),
            categories: None,
            newest_item_publish_time: None,
            trend_score: None,
        };

        let podcast = podcast_from_feed(feed);
        assert_eq!(
            podcast.artwork_url,
            Some("https://example.com/image.jpg".to_string())
        );
    }

    #[test]
    fn test_episode_from_podcast_episode() {
        let podcast_episode = PodcastEpisode {
            id: 123456,
            title: "Test Episode".to_string(),
            description: Some("Episode description".to_string()),
            guid: "guid-123".to_string(),
            link: Some("https://example.com/episode".to_string()),
            date_published: 1634567890,
            date_published_pretty: Some("Oct 18, 2021".to_string()),
            date_crawled: 1634568000,
            enclosure_url: "https://example.com/audio.mp3".to_string(),
            enclosure_type: "audio/mpeg".to_string(),
            enclosure_length: 12345678,
            duration: Some(3600),
            explicit: 1,
            episode: Some(42),
            season: Some(2),
            episode_type: Some("full".to_string()),
            image: Some("https://example.com/episode-image.jpg".to_string()),
            feed_id: 6974466,
            feed_title: Some("Test Podcast".to_string()),
            feed_image: Some("https://example.com/feed-image.jpg".to_string()),
            feed_itunes_id: Some(123456),
            feed_language: Some("en".to_string()),
            feed_url: Some("https://example.com/feed.xml".to_string()),
            feed_dead: None,
            feed_duplicate_of: None,
            chapters_url: Some("https://example.com/chapters.json".to_string()),
            transcript_url: Some("https://example.com/transcript.vtt".to_string()),
            podcast_guid: Some("podcast-guid-123".to_string()),
        };

        let episode = episode_from_podcast_episode(podcast_episode);

        assert_eq!(episode.id, 123456);
        assert_eq!(episode.podcast_id, 6974466);
        assert_eq!(episode.title, "Test Episode");
        assert_eq!(episode.description, Some("Episode description".to_string()));
        assert_eq!(episode.audio_url, "https://example.com/audio.mp3");
        assert_eq!(episode.audio_type, Some("audio/mpeg".to_string()));
        assert_eq!(episode.audio_length, Some(12345678));
        assert_eq!(episode.duration_seconds, Some(3600));
        assert_eq!(episode.episode_number, Some(42));
        assert_eq!(episode.season_number, Some(2));
        assert_eq!(episode.episode_type, Some("full".to_string()));
        assert_eq!(episode.explicit, Some(true));
        assert!(episode.published_at.is_some());
    }

    #[test]
    fn test_episode_explicit_false() {
        let podcast_episode = PodcastEpisode {
            id: 1,
            title: "Test".to_string(),
            description: None,
            guid: "guid".to_string(),
            link: None,
            date_published: 1634567890,
            date_published_pretty: None,
            date_crawled: 1634568000,
            enclosure_url: "https://example.com/audio.mp3".to_string(),
            enclosure_type: "audio/mpeg".to_string(),
            enclosure_length: 12345678,
            duration: None,
            explicit: 0,
            episode: None,
            season: None,
            episode_type: None,
            image: None,
            feed_id: 1,
            feed_title: None,
            feed_image: None,
            feed_itunes_id: None,
            feed_language: None,
            feed_url: None,
            feed_dead: None,
            feed_duplicate_of: None,
            chapters_url: None,
            transcript_url: None,
            podcast_guid: None,
        };

        let episode = episode_from_podcast_episode(podcast_episode);
        assert_eq!(episode.explicit, Some(false));
    }

    #[test]
    fn test_podcast_categories_parsing() {
        let feed = PodcastFeed {
            id: 1,
            title: "Test".to_string(),
            author: "Author".to_string(),
            description: None,
            image: None,
            artwork: None,
            url: "https://example.com/feed.xml".to_string(),
            itunes_id: None,
            language: "en".to_string(),
            categories: Some(serde_json::json!({
                "102": "Technology",
                "103": "News",
                "invalid": "Should be skipped"
            })),
            newest_item_publish_time: None,
            trend_score: None,
        };

        let podcast = podcast_from_feed(feed);
        assert_eq!(podcast.categories.len(), 2);

        let tech_cat = podcast.categories.iter().find(|c| c.id == 102);
        assert!(tech_cat.is_some());
        assert_eq!(tech_cat.unwrap().name, "Technology");
    }

    #[test]
    fn test_podcast_empty_categories() {
        let feed = PodcastFeed {
            id: 1,
            title: "Test".to_string(),
            author: "Author".to_string(),
            description: None,
            image: None,
            artwork: None,
            url: "https://example.com/feed.xml".to_string(),
            itunes_id: None,
            language: "en".to_string(),
            categories: None,
            newest_item_publish_time: None,
            trend_score: None,
        };

        let podcast = podcast_from_feed(feed);
        assert_eq!(podcast.categories.len(), 0);
    }
}
