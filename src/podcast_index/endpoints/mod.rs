//! PodcastIndex API endpoint implementations.

pub mod categories;
pub mod episodes;
pub mod podcasts;
pub mod search;
pub mod trending;

pub use categories::get_categories;
pub use episodes::{get_episode_by_id, get_episodes, get_random_episodes};
pub use podcasts::get_podcast_by_feed_id;
pub use search::{search_by_author, search_by_title, search_podcasts};
pub use trending::get_trending;
