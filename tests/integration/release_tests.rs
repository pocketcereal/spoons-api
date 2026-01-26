//! Integration tests for release caching.

use spoons_api::db::repositories::ReleaseRepository;

use crate::common::{TestDb, nevermind_release, ok_computer_release};

#[tokio::test]
async fn test_release_upsert_and_get() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let release = nevermind_release();

    // Insert release
    ReleaseRepository::upsert(&test_db.pool, &release)
        .await
        .expect("Failed to upsert release");

    // Retrieve release (with long TTL)
    let cached = ReleaseRepository::get_cached(&test_db.pool, &release.id, 86400)
        .await
        .expect("Failed to get cached release");

    assert!(cached.is_some());
    let cached_release = cached.unwrap();
    assert_eq!(cached_release.id, release.id);
    assert_eq!(cached_release.title, release.title);
    assert_eq!(cached_release.status, release.status);
    assert_eq!(cached_release.date, release.date);
    assert_eq!(cached_release.country, release.country);

    // Verify release group is present
    assert!(cached_release.release_group.is_some());
    let rg = cached_release.release_group.unwrap();
    assert_eq!(rg.title, "Nevermind");
    assert_eq!(rg.primary_type, Some("Album".to_string()));
}

#[tokio::test]
async fn test_release_cache_expiry() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let release = nevermind_release();

    // Insert release
    ReleaseRepository::upsert(&test_db.pool, &release)
        .await
        .expect("Failed to upsert release");

    // Try to get with -1 TTL (should return None as it's immediately expired)
    // We use -1 instead of 0 to avoid clock skew issues between DB and app
    let cached = ReleaseRepository::get_cached(&test_db.pool, &release.id, -1)
        .await
        .expect("Failed to get cached release");

    assert!(
        cached.is_none(),
        "Cache should be expired with negative TTL"
    );
}

#[tokio::test]
async fn test_release_get_by_ids() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let nevermind = nevermind_release();
    let ok_computer = ok_computer_release();

    // Insert both releases
    ReleaseRepository::upsert(&test_db.pool, &nevermind)
        .await
        .expect("Failed to upsert Nevermind");
    ReleaseRepository::upsert(&test_db.pool, &ok_computer)
        .await
        .expect("Failed to upsert OK Computer");

    // Get by IDs
    let ids: Vec<uuid::Uuid> = vec![
        uuid::Uuid::parse_str(&nevermind.id).unwrap(),
        uuid::Uuid::parse_str(&ok_computer.id).unwrap(),
    ];

    let releases = ReleaseRepository::get_by_ids(&test_db.pool, &ids)
        .await
        .expect("Failed to get releases by IDs");

    assert_eq!(releases.len(), 2);

    let titles: Vec<&str> = releases.iter().map(|r| r.title.as_str()).collect();
    assert!(titles.contains(&"Nevermind"));
    assert!(titles.contains(&"OK Computer"));
}

#[tokio::test]
async fn test_release_upsert_updates_release_group() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let mut release = nevermind_release();

    // Insert release
    ReleaseRepository::upsert(&test_db.pool, &release)
        .await
        .expect("Failed to upsert release");

    // Update the release group info
    if let Some(ref mut rg) = release.release_group {
        rg.disambiguation = Some("Remastered edition".to_string());
    }

    ReleaseRepository::upsert(&test_db.pool, &release)
        .await
        .expect("Failed to upsert updated release");

    // Verify update
    let retrieved = ReleaseRepository::get_by_id(&test_db.pool, &release.id)
        .await
        .expect("Failed to get release")
        .expect("Release should exist");

    let rg = retrieved.release_group.expect("Release group should exist");
    assert_eq!(rg.disambiguation, Some("Remastered edition".to_string()));
}
