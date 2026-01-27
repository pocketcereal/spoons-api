//! Trending podcasts endpoint implementation.

use crate::error::Result;
use crate::podcast::Podcast;
use crate::podcast_index::client::PodcastIndexClient;
use crate::podcast_index::conversions::podcast_from_feed;
use crate::podcast_index::types::PodcastIndexResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct TrendingParams {
    max: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    cat: Option<String>,
}

/// Gets trending podcasts using the PodcastIndex `/podcasts/trending` endpoint.
///
/// # Arguments
/// * `client` - The PodcastIndex client
/// * `limit` - Maximum number of results (capped at 100)
/// * `categories` - Optional category IDs to filter by
pub async fn get_trending(
    client: &PodcastIndexClient,
    limit: i32,
    categories: Option<&[i32]>,
) -> Result<Vec<Podcast>> {
    let cat = categories.map(|cats| {
        cats.iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",")
    });

    let params = TrendingParams {
        max: limit.min(100),
        cat,
    };

    let response: PodcastIndexResponse<crate::podcast_index::types::PodcastFeed> =
        client.get_with_query("/podcasts/trending", &params).await?;

    let feeds = response.feeds.unwrap_or_default();
    Ok(feeds.into_iter().map(podcast_from_feed).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trending_params_serialization() {
        let params = TrendingParams {
            max: 25,
            cat: Some("102,103".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["max"], 25);
        assert_eq!(json["cat"], "102,103");
    }

    #[test]
    fn test_trending_params_no_categories() {
        let params = TrendingParams { max: 10, cat: None };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["max"], 10);
        assert!(json.get("cat").is_none());
    }
}
