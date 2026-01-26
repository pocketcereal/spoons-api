//! Integration tests for artist caching.

use spoons_api::db::repositories::ArtistRepository;

use crate::common::{TestDb, nirvana_artist, radiohead_artist};

#[tokio::test]
async fn test_artist_upsert_and_get() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let artist = nirvana_artist();

    // Insert artist
    ArtistRepository::upsert(&test_db.pool, &artist)
        .await
        .expect("Failed to upsert artist");

    // Retrieve artist (with long TTL so it's always valid)
    let cached = ArtistRepository::get_cached(&test_db.pool, &artist.id, 86400)
        .await
        .expect("Failed to get cached artist");

    assert!(cached.is_some());
    let cached_artist = cached.unwrap();
    assert_eq!(cached_artist.id, artist.id);
    assert_eq!(cached_artist.name, artist.name);
    assert_eq!(cached_artist.sort_name, artist.sort_name);
    assert_eq!(cached_artist.artist_type, artist.artist_type);
    assert_eq!(cached_artist.country, artist.country);

    // Verify area is present
    assert!(cached_artist.area.is_some());
    let area = cached_artist.area.unwrap();
    assert_eq!(area.name, "United States");

    // Verify life span
    assert!(cached_artist.life_span.is_some());
    let life_span = cached_artist.life_span.unwrap();
    assert_eq!(life_span.begin, Some("1987".to_string()));
    assert_eq!(life_span.ended, Some(true));
}

#[tokio::test]
async fn test_artist_cache_expiry() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let artist = nirvana_artist();

    // Insert artist
    ArtistRepository::upsert(&test_db.pool, &artist)
        .await
        .expect("Failed to upsert artist");

    // Try to get with -1 TTL (should return None as it's immediately expired)
    // We use -1 instead of 0 to avoid clock skew issues between DB and app
    let cached = ArtistRepository::get_cached(&test_db.pool, &artist.id, -1)
        .await
        .expect("Failed to get cached artist");

    assert!(
        cached.is_none(),
        "Cache should be expired with negative TTL"
    );
}

#[tokio::test]
async fn test_artist_get_by_id() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let artist = nirvana_artist();

    // Insert artist
    ArtistRepository::upsert(&test_db.pool, &artist)
        .await
        .expect("Failed to upsert artist");

    // Get by ID (ignoring cache expiry)
    let retrieved = ArtistRepository::get_by_id(&test_db.pool, &artist.id)
        .await
        .expect("Failed to get artist by ID");

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Nirvana");
}

#[tokio::test]
async fn test_artist_get_by_ids() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let nirvana = nirvana_artist();
    let radiohead = radiohead_artist();

    // Insert both artists
    ArtistRepository::upsert(&test_db.pool, &nirvana)
        .await
        .expect("Failed to upsert Nirvana");
    ArtistRepository::upsert(&test_db.pool, &radiohead)
        .await
        .expect("Failed to upsert Radiohead");

    // Get by IDs
    let ids: Vec<uuid::Uuid> = vec![
        uuid::Uuid::parse_str(&nirvana.id).unwrap(),
        uuid::Uuid::parse_str(&radiohead.id).unwrap(),
    ];

    let artists = ArtistRepository::get_by_ids(&test_db.pool, &ids)
        .await
        .expect("Failed to get artists by IDs");

    assert_eq!(artists.len(), 2);

    let names: Vec<&str> = artists.iter().map(|a| a.name.as_str()).collect();
    assert!(names.contains(&"Nirvana"));
    assert!(names.contains(&"Radiohead"));
}

#[tokio::test]
async fn test_artist_upsert_updates_existing() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let mut artist = nirvana_artist();

    // Insert artist
    ArtistRepository::upsert(&test_db.pool, &artist)
        .await
        .expect("Failed to upsert artist");

    // Update the artist
    artist.disambiguation = Some("Updated disambiguation".to_string());

    ArtistRepository::upsert(&test_db.pool, &artist)
        .await
        .expect("Failed to upsert updated artist");

    // Verify update
    let retrieved = ArtistRepository::get_by_id(&test_db.pool, &artist.id)
        .await
        .expect("Failed to get artist")
        .expect("Artist should exist");

    assert_eq!(
        retrieved.disambiguation,
        Some("Updated disambiguation".to_string())
    );
}

#[tokio::test]
async fn test_artist_not_found() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let non_existent_id = "00000000-0000-0000-0000-000000000000";

    let result = ArtistRepository::get_by_id(&test_db.pool, non_existent_id)
        .await
        .expect("Failed to query");

    assert!(result.is_none());
}
