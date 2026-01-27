//! Podcast by ID endpoint implementation.

use crate::error::Result;
use crate::podcast::Podcast;
use crate::podcast_index::client::PodcastIndexClient;
use crate::podcast_index::conversions::podcast_from_feed;
use crate::podcast_index::types::PodcastIndexPodcastResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct PodcastByIdParams {
    id: i64,
}

/// Gets a podcast by its feed ID using the PodcastIndex `/podcasts/byfeedid` endpoint.
///
/// # Arguments
/// * `client` - The PodcastIndex client
/// * `feed_id` - The PodcastIndex feed ID
pub async fn get_podcast_by_feed_id(client: &PodcastIndexClient, feed_id: i64) -> Result<Podcast> {
    let params = PodcastByIdParams { id: feed_id };

    let response: PodcastIndexPodcastResponse =
        client.get_with_query("/podcasts/byfeedid", &params).await?;

    Ok(podcast_from_feed(response.feed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_podcast_by_id_params_serialization() {
        let params = PodcastByIdParams { id: 6974466 };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["id"], 6974466);
    }
}
