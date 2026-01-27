//! Integration tests for podcast GraphQL queries.
//!
//! These tests verify the full stack: GraphQL -> Repository -> Database.
//! They use seeded data and do NOT make external API calls.

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use spoons_api::db::repositories::{EpisodeRepository, PodcastRepository, SearchCacheRepository};
use spoons_api::graphql::{AppContext, QueryRoot};

use crate::common::{
    TestDb, daily_episode, syntax_episode_1, syntax_episode_2, syntax_podcast, the_daily_podcast,
};

async fn setup_podcast_test() -> (TestDb, Schema<QueryRoot, EmptyMutation, EmptySubscription>) {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    // Note: podcast_index_client is None in tests (that's OK - we'll seed data manually)
    let app_context = AppContext {
        db_pool: test_db.pool.clone(),
        musicbrainz_client: spoons_api::musicbrainz::MusicBrainzClient::new(
            "https://musicbrainz.org/ws/2",
        )
        .expect("Failed to create MusicBrainz client"),
        audius_client: None,
        podcast_index_client: None,
        cache_ttl_seconds: 86400, // Long TTL for tests
    };

    let schema = Schema::build(QueryRoot::default(), EmptyMutation, EmptySubscription)
        .data(std::sync::Arc::new(app_context))
        .finish();

    (test_db, schema)
}

#[tokio::test]
async fn test_podcast_upsert_and_retrieve() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let podcast = syntax_podcast();
    PodcastRepository::upsert(&test_db.pool, &podcast)
        .await
        .expect("Failed to upsert podcast");

    let retrieved = PodcastRepository::get_by_id(&test_db.pool, podcast.id)
        .await
        .expect("Failed to retrieve podcast");

    assert!(retrieved.is_some(), "Podcast should be found");
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, podcast.id);
    assert_eq!(retrieved.title, podcast.title);
    assert_eq!(retrieved.author, podcast.author);
    assert_eq!(retrieved.feed_url, podcast.feed_url);
    assert_eq!(retrieved.itunes_id, podcast.itunes_id);
}

#[tokio::test]
async fn test_episode_upsert_and_retrieve() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    // First, upsert the parent podcast
    let podcast = syntax_podcast();
    PodcastRepository::upsert(&test_db.pool, &podcast)
        .await
        .expect("Failed to upsert podcast");

    // Then upsert the episode
    let episode = syntax_episode_1();
    EpisodeRepository::upsert(&test_db.pool, &episode)
        .await
        .expect("Failed to upsert episode");

    let retrieved = EpisodeRepository::get_by_id(&test_db.pool, episode.id)
        .await
        .expect("Failed to retrieve episode");

    assert!(retrieved.is_some(), "Episode should be found");
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, episode.id);
    assert_eq!(retrieved.podcast_id, episode.podcast_id);
    assert_eq!(retrieved.title, episode.title);
    assert_eq!(retrieved.audio_url, episode.audio_url);
    assert_eq!(retrieved.episode_number, episode.episode_number);
}

#[tokio::test]
async fn test_get_episodes_by_podcast_id() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    // Seed podcast and two episodes
    let podcast = syntax_podcast();
    PodcastRepository::upsert(&test_db.pool, &podcast)
        .await
        .unwrap();

    let episode1 = syntax_episode_1();
    let episode2 = syntax_episode_2();
    EpisodeRepository::upsert(&test_db.pool, &episode1)
        .await
        .unwrap();
    EpisodeRepository::upsert(&test_db.pool, &episode2)
        .await
        .unwrap();

    // Retrieve episodes by podcast ID
    let episodes = EpisodeRepository::get_by_podcast_id(&test_db.pool, podcast.id, 10)
        .await
        .expect("Failed to get episodes by podcast ID");

    assert_eq!(episodes.len(), 2, "Should retrieve 2 episodes");
    let episode_ids: Vec<i64> = episodes.iter().map(|e| e.id).collect();
    assert!(episode_ids.contains(&episode1.id));
    assert!(episode_ids.contains(&episode2.id));
}

#[tokio::test]
async fn test_podcast_cache_expiry() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let podcast = syntax_podcast();
    PodcastRepository::upsert(&test_db.pool, &podcast)
        .await
        .unwrap();

    // With a very long TTL, should find the cached podcast
    let cached_fresh = PodcastRepository::get_cached(&test_db.pool, podcast.id, 86400)
        .await
        .expect("Failed to get cached podcast");
    assert!(cached_fresh.is_some(), "Should find fresh cached podcast");

    // With a TTL of -1 seconds, the cache is immediately expired
    // We use -1 instead of 0 to avoid clock skew issues between DB and app
    let cached_expired = PodcastRepository::get_cached(&test_db.pool, podcast.id, -1)
        .await
        .expect("Failed to get cached podcast");
    assert!(
        cached_expired.is_none(),
        "Should not find expired cached podcast"
    );
}

#[tokio::test]
async fn test_episode_cache_expiry() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let podcast = syntax_podcast();
    PodcastRepository::upsert(&test_db.pool, &podcast)
        .await
        .unwrap();

    let episode = syntax_episode_1();
    EpisodeRepository::upsert(&test_db.pool, &episode)
        .await
        .unwrap();

    // With a long TTL, should find the cached episode
    let cached_fresh = EpisodeRepository::get_cached(&test_db.pool, episode.id, 86400)
        .await
        .expect("Failed to get cached episode");
    assert!(cached_fresh.is_some(), "Should find fresh cached episode");

    // With a TTL of -1 seconds, the cache is immediately expired
    // We use -1 instead of 0 to avoid clock skew issues between DB and app
    let cached_expired = EpisodeRepository::get_cached(&test_db.pool, episode.id, -1)
        .await
        .expect("Failed to get cached episode");
    assert!(
        cached_expired.is_none(),
        "Should not find expired cached episode"
    );
}

#[tokio::test]
async fn test_podcast_search_cache() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let syntax = syntax_podcast();
    let daily = the_daily_podcast();

    // Cache the search results
    let query = "web development";
    let podcasts = vec![syntax.clone(), daily.clone()];
    SearchCacheRepository::cache_podcast_search(&test_db.pool, query, 10, 0, &podcasts)
        .await
        .expect("Failed to cache podcast search");

    // Verify the search was cached
    let cached = SearchCacheRepository::get_podcast_search(&test_db.pool, query, 10, 0, 86400)
        .await
        .expect("Failed to get cached podcast search");

    assert!(cached.is_some(), "Search results should be cached");
    let cached_podcasts = cached.unwrap();
    assert_eq!(cached_podcasts.len(), 2, "Should have 2 cached podcasts");

    let cached_ids: Vec<i64> = cached_podcasts.iter().map(|p| p.id).collect();
    assert!(cached_ids.contains(&syntax.id));
    assert!(cached_ids.contains(&daily.id));
}

#[tokio::test]
async fn test_podcast_search_cache_expiry() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let syntax = syntax_podcast();
    let query = "web development";
    let podcasts = vec![syntax];

    // Cache the search
    SearchCacheRepository::cache_podcast_search(&test_db.pool, query, 10, 0, &podcasts)
        .await
        .unwrap();

    // With long TTL, should find cached results
    let cached_fresh =
        SearchCacheRepository::get_podcast_search(&test_db.pool, query, 10, 0, 86400)
            .await
            .expect("Failed to get cached search");
    assert!(
        cached_fresh.is_some(),
        "Should find fresh cached search results"
    );

    // With TTL of -1, should not find cached results
    // We use -1 instead of 0 to avoid clock skew issues between DB and app
    let cached_expired = SearchCacheRepository::get_podcast_search(&test_db.pool, query, 10, 0, -1)
        .await
        .expect("Failed to get cached search");
    assert!(
        cached_expired.is_none(),
        "Should not find expired cached search results"
    );
}

#[tokio::test]
async fn test_podcast_batch_upsert() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let syntax = syntax_podcast();
    let daily = the_daily_podcast();
    let podcasts = vec![syntax.clone(), daily.clone()];

    // Batch upsert
    PodcastRepository::upsert_many(&test_db.pool, &podcasts)
        .await
        .expect("Failed to batch upsert podcasts");

    // Verify both were inserted
    let retrieved_syntax = PodcastRepository::get_by_id(&test_db.pool, syntax.id)
        .await
        .unwrap();
    let retrieved_daily = PodcastRepository::get_by_id(&test_db.pool, daily.id)
        .await
        .unwrap();

    assert!(retrieved_syntax.is_some());
    assert!(retrieved_daily.is_some());
}

#[tokio::test]
async fn test_episode_batch_upsert() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    // Seed the parent podcasts
    let syntax = syntax_podcast();
    let daily = the_daily_podcast();
    PodcastRepository::upsert_many(&test_db.pool, &[syntax, daily])
        .await
        .unwrap();

    // Batch upsert episodes
    let episode1 = syntax_episode_1();
    let episode2 = syntax_episode_2();
    let episode3 = daily_episode();
    let episodes = vec![episode1.clone(), episode2.clone(), episode3.clone()];

    EpisodeRepository::upsert_many(&test_db.pool, &episodes)
        .await
        .expect("Failed to batch upsert episodes");

    // Verify all were inserted
    let retrieved1 = EpisodeRepository::get_by_id(&test_db.pool, episode1.id)
        .await
        .unwrap();
    let retrieved2 = EpisodeRepository::get_by_id(&test_db.pool, episode2.id)
        .await
        .unwrap();
    let retrieved3 = EpisodeRepository::get_by_id(&test_db.pool, episode3.id)
        .await
        .unwrap();

    assert!(retrieved1.is_some());
    assert!(retrieved2.is_some());
    assert!(retrieved3.is_some());
}

#[tokio::test]
async fn test_podcast_get_by_ids() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let syntax = syntax_podcast();
    let daily = the_daily_podcast();

    PodcastRepository::upsert(&test_db.pool, &syntax)
        .await
        .unwrap();
    PodcastRepository::upsert(&test_db.pool, &daily)
        .await
        .unwrap();

    // Get both podcasts by IDs
    let ids = vec![syntax.id, daily.id];
    let podcasts = PodcastRepository::get_by_ids(&test_db.pool, &ids)
        .await
        .expect("Failed to get podcasts by IDs");

    assert_eq!(podcasts.len(), 2);
    let podcast_ids: Vec<i64> = podcasts.iter().map(|p| p.id).collect();
    assert!(podcast_ids.contains(&syntax.id));
    assert!(podcast_ids.contains(&daily.id));
}

#[tokio::test]
async fn test_seeded_podcast_data_retrieval() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    // Seed all podcast test data
    let syntax = syntax_podcast();
    let daily = the_daily_podcast();
    let episode1 = syntax_episode_1();
    let episode2 = syntax_episode_2();
    let episode3 = daily_episode();

    // Insert all data
    PodcastRepository::upsert(&test_db.pool, &syntax)
        .await
        .unwrap();
    PodcastRepository::upsert(&test_db.pool, &daily)
        .await
        .unwrap();
    EpisodeRepository::upsert(&test_db.pool, &episode1)
        .await
        .unwrap();
    EpisodeRepository::upsert(&test_db.pool, &episode2)
        .await
        .unwrap();
    EpisodeRepository::upsert(&test_db.pool, &episode3)
        .await
        .unwrap();

    // Verify all data can be retrieved
    let podcasts = PodcastRepository::get_by_ids(&test_db.pool, &[syntax.id, daily.id])
        .await
        .expect("Failed to get podcasts");
    assert_eq!(podcasts.len(), 2);

    let syntax_episodes = EpisodeRepository::get_by_podcast_id(&test_db.pool, syntax.id, 10)
        .await
        .expect("Failed to get syntax episodes");
    assert_eq!(syntax_episodes.len(), 2);

    let daily_episodes = EpisodeRepository::get_by_podcast_id(&test_db.pool, daily.id, 10)
        .await
        .expect("Failed to get daily episodes");
    assert_eq!(daily_episodes.len(), 1);
}

#[tokio::test]
async fn test_graphql_version_query() {
    let (_test_db, schema) = setup_podcast_test().await;

    let query = r#"
        query {
            version
        }
    "#;

    let result = schema.execute(query).await;
    assert!(
        result.errors.is_empty(),
        "Query should succeed: {:?}",
        result.errors
    );

    let data = result.data.into_json().expect("Failed to convert to JSON");
    let version = data["version"]
        .as_str()
        .expect("Version should be a string");
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}
