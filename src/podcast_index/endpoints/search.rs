//! Search endpoint implementations.

use crate::error::Result;
use crate::podcast::Podcast;
use crate::podcast_index::client::PodcastIndexClient;
use crate::podcast_index::conversions::podcast_from_feed;
use crate::podcast_index::types::PodcastIndexListResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct SearchParams {
    q: String,
    max: i32,
}

/// Searches for podcasts using the PodcastIndex `/search/byterm` endpoint.
///
/// This searches across title, author, owner, and description fields.
pub async fn search_podcasts(
    client: &PodcastIndexClient,
    query: &str,
    limit: i32,
) -> Result<Vec<Podcast>> {
    let params = SearchParams {
        q: query.to_string(),
        max: limit.min(1000),
    };

    let response: PodcastIndexListResponse<crate::podcast_index::types::PodcastFeed> =
        client.get_with_query("/search/byterm", &params).await?;

    let feeds = response.feeds.unwrap_or_default();
    Ok(feeds.into_iter().map(podcast_from_feed).collect())
}

/// Searches for podcasts by title using the PodcastIndex `/search/bytitle` endpoint.
///
/// This searches only in the podcast title field for more precise results.
pub async fn search_by_title(
    client: &PodcastIndexClient,
    title: &str,
    limit: i32,
) -> Result<Vec<Podcast>> {
    let params = SearchParams {
        q: title.to_string(),
        max: limit.min(1000),
    };

    let response: PodcastIndexListResponse<crate::podcast_index::types::PodcastFeed> =
        client.get_with_query("/search/bytitle", &params).await?;

    let feeds = response.feeds.unwrap_or_default();
    Ok(feeds.into_iter().map(podcast_from_feed).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_params_serialization() {
        let params = SearchParams {
            q: "rust".to_string(),
            max: 10,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["q"], "rust");
        assert_eq!(json["max"], 10);
    }
}
