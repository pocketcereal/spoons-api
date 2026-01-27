//! Categories endpoint implementation.

use crate::error::Result;
use crate::podcast::Category;
use crate::podcast_index::client::PodcastIndexClient;
use crate::podcast_index::types::PodcastIndexResponse;

/// Gets all available podcast categories using the PodcastIndex `/categories/list` endpoint.
pub async fn get_categories(client: &PodcastIndexClient) -> Result<Vec<Category>> {
    let response: PodcastIndexResponse<crate::podcast_index::types::Category> =
        client.get("/categories/list").await?;

    let categories = response.feeds.unwrap_or_default();
    Ok(categories
        .into_iter()
        .map(|c| Category {
            id: c.id,
            name: c.name,
        })
        .collect())
}
