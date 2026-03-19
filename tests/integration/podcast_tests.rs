//! Full-stack integration tests (GraphQL -> Repository -> Database).
//! Uses seeded data, no external API calls.

use spoons_api::db::repositories::{EpisodeRepository, PodcastRepository, SearchCacheRepository};

use crate::common::{
    TestDb, daily_episode, syntax_episode_1, syntax_episode_2, syntax_podcast, the_daily_podcast,
};

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

    let podcast = syntax_podcast();
    PodcastRepository::upsert(&test_db.pool, &podcast)
        .await
        .expect("Failed to upsert podcast");

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

    let cached_fresh = PodcastRepository::get_cached(&test_db.pool, podcast.id, 86400)
        .await
        .expect("Failed to get cached podcast");
    assert!(cached_fresh.is_some(), "Should find fresh cached podcast");

    // -1 instead of 0 to avoid clock skew between DB and app
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

    let cached_fresh = EpisodeRepository::get_cached(&test_db.pool, episode.id, 86400)
        .await
        .expect("Failed to get cached episode");
    assert!(cached_fresh.is_some(), "Should find fresh cached episode");

    // -1 instead of 0 to avoid clock skew between DB and app
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

    let query = "web development";
    let podcasts = vec![syntax.clone(), daily.clone()];
    SearchCacheRepository::cache_podcast_search(&test_db.pool, query, 10, 0, &podcasts)
        .await
        .expect("Failed to cache podcast search");

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

    SearchCacheRepository::cache_podcast_search(&test_db.pool, query, 10, 0, &podcasts)
        .await
        .unwrap();

    let cached_fresh =
        SearchCacheRepository::get_podcast_search(&test_db.pool, query, 10, 0, 86400)
            .await
            .expect("Failed to get cached search");
    assert!(
        cached_fresh.is_some(),
        "Should find fresh cached search results"
    );

    // -1 instead of 0 to avoid clock skew between DB and app
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

    PodcastRepository::upsert_many(&test_db.pool, &podcasts)
        .await
        .expect("Failed to batch upsert podcasts");

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

    let syntax = syntax_podcast();
    let daily = the_daily_podcast();
    PodcastRepository::upsert_many(&test_db.pool, &[syntax, daily])
        .await
        .unwrap();

    let episode1 = syntax_episode_1();
    let episode2 = syntax_episode_2();
    let episode3 = daily_episode();
    let episodes = vec![episode1.clone(), episode2.clone(), episode3.clone()];

    EpisodeRepository::upsert_many(&test_db.pool, &episodes)
        .await
        .expect("Failed to batch upsert episodes");

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

    let syntax = syntax_podcast();
    let daily = the_daily_podcast();
    let episode1 = syntax_episode_1();
    let episode2 = syntax_episode_2();
    let episode3 = daily_episode();

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
