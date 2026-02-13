//! Episodes endpoint implementations.

use crate::error::Result;
use crate::podcast::Episode;
use crate::podcast_index::client::PodcastIndexClient;
use crate::podcast_index::conversions::episode_from_podcast_episode;
use crate::podcast_index::types::{
    PodcastIndexEpisodeByIdResponse, PodcastIndexEpisodesResponse,
    PodcastIndexRandomEpisodesResponse,
};
use serde::Serialize;

use super::format_category_list;

#[derive(Debug, Serialize)]
struct EpisodesByFeedParams {
    id: i64,
    max: i32,
}

#[derive(Debug, Serialize)]
struct EpisodeByIdParams {
    id: i64,
}

#[derive(Debug, Serialize)]
struct RandomEpisodesParams {
    max: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cat: Option<String>,
}

/// Gets episodes for a podcast using the PodcastIndex `/episodes/byfeedid` endpoint.
///
/// # Arguments
/// * `client` - The PodcastIndex client
/// * `feed_id` - The PodcastIndex feed ID
/// * `limit` - Maximum number of episodes to return (capped at 1000)
pub async fn get_episodes(
    client: &PodcastIndexClient,
    feed_id: i64,
    limit: i32,
) -> Result<Vec<Episode>> {
    let params = EpisodesByFeedParams {
        id: feed_id,
        max: limit.min(1000),
    };

    let response: PodcastIndexEpisodesResponse =
        client.get_with_query("/episodes/byfeedid", &params).await?;

    Ok(response
        .items
        .into_iter()
        .map(episode_from_podcast_episode)
        .collect())
}

/// Gets a single episode by its ID using the PodcastIndex `/episodes/byid` endpoint.
///
/// # Arguments
/// * `client` - The PodcastIndex client
/// * `episode_id` - The PodcastIndex episode ID
pub async fn get_episode_by_id(client: &PodcastIndexClient, episode_id: i64) -> Result<Episode> {
    let params = EpisodeByIdParams { id: episode_id };

    let response: PodcastIndexEpisodeByIdResponse =
        client.get_with_query("/episodes/byid", &params).await?;

    Ok(episode_from_podcast_episode(response.episode))
}

/// Gets random episodes using the PodcastIndex `/episodes/random` endpoint.
///
/// # Arguments
/// * `client` - The PodcastIndex client
/// * `limit` - Maximum number of episodes to return (capped at 40)
/// * `lang` - Optional language filter (e.g., "en", "es")
/// * `categories` - Optional category IDs to filter by
pub async fn get_random_episodes(
    client: &PodcastIndexClient,
    limit: i32,
    lang: Option<&str>,
    categories: Option<&[i32]>,
) -> Result<Vec<Episode>> {
    let params = RandomEpisodesParams {
        max: limit.min(40),
        lang: lang.map(String::from),
        cat: format_category_list(categories),
    };

    let response: PodcastIndexRandomEpisodesResponse =
        client.get_with_query("/episodes/random", &params).await?;

    Ok(response
        .episodes
        .into_iter()
        .map(episode_from_podcast_episode)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episodes_by_feed_params_serialization() {
        let params = EpisodesByFeedParams {
            id: 6974466,
            max: 50,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["id"], 6974466);
        assert_eq!(json["max"], 50);
    }

    #[test]
    fn test_episode_by_id_params_serialization() {
        let params = EpisodeByIdParams { id: 123456789 };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["id"], 123456789);
    }

    #[test]
    fn test_random_episodes_params_serialization() {
        let params = RandomEpisodesParams {
            max: 20,
            lang: Some("en".to_string()),
            cat: Some("102,103".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["max"], 20);
        assert_eq!(json["lang"], "en");
        assert_eq!(json["cat"], "102,103");
    }

    #[test]
    fn test_random_episodes_params_no_filters() {
        let params = RandomEpisodesParams {
            max: 10,
            lang: None,
            cat: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["max"], 10);
        assert!(json.get("lang").is_none());
        assert!(json.get("cat").is_none());
    }
}
