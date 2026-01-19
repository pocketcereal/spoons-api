//! Integration tests for release group caching.

use spoons_api::db::repositories::ReleaseGroupRepository;

use crate::common::{TestDb, nevermind_release_group, ok_computer_release_group};

#[tokio::test]
async fn test_release_group_upsert_and_get() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let release_group = nevermind_release_group();

    // Insert release group
    ReleaseGroupRepository::upsert(&test_db.pool, &release_group)
        .await
        .expect("Failed to upsert release group");

    // Retrieve release group (with long TTL)
    let cached = ReleaseGroupRepository::get_cached(&test_db.pool, &release_group.id, 86400)
        .await
        .expect("Failed to get cached release group");

    assert!(cached.is_some());
    let cached_rg = cached.unwrap();
    assert_eq!(cached_rg.id, release_group.id);
    assert_eq!(cached_rg.title, release_group.title);
    assert_eq!(cached_rg.primary_type, release_group.primary_type);
    assert_eq!(cached_rg.first_release_date, release_group.first_release_date);
}

#[tokio::test]
async fn test_release_group_cache_expiry() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let release_group = nevermind_release_group();

    // Insert release group
    ReleaseGroupRepository::upsert(&test_db.pool, &release_group)
        .await
        .expect("Failed to upsert release group");

    // Try to get with -1 TTL (should return None as it's immediately expired)
    // We use -1 instead of 0 to avoid clock skew issues between DB and app
    let cached = ReleaseGroupRepository::get_cached(&test_db.pool, &release_group.id, -1)
        .await
        .expect("Failed to get cached release group");

    assert!(cached.is_none(), "Cache should be expired with negative TTL");
}

#[tokio::test]
async fn test_release_group_get_by_ids() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let nevermind = nevermind_release_group();
    let ok_computer = ok_computer_release_group();

    // Insert both release groups
    ReleaseGroupRepository::upsert(&test_db.pool, &nevermind)
        .await
        .expect("Failed to upsert Nevermind");
    ReleaseGroupRepository::upsert(&test_db.pool, &ok_computer)
        .await
        .expect("Failed to upsert OK Computer");

    // Get by IDs
    let ids: Vec<uuid::Uuid> = vec![
        uuid::Uuid::parse_str(&nevermind.id).unwrap(),
        uuid::Uuid::parse_str(&ok_computer.id).unwrap(),
    ];

    let release_groups = ReleaseGroupRepository::get_by_ids(&test_db.pool, &ids)
        .await
        .expect("Failed to get release groups by IDs");

    assert_eq!(release_groups.len(), 2);

    let titles: Vec<&str> = release_groups.iter().map(|rg| rg.title.as_str()).collect();
    assert!(titles.contains(&"Nevermind"));
    assert!(titles.contains(&"OK Computer"));
}

#[tokio::test]
async fn test_release_group_with_secondary_types() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let mut release_group = nevermind_release_group();
    release_group.secondary_types = Some(vec!["Compilation".to_string(), "Live".to_string()]);

    // Insert release group
    ReleaseGroupRepository::upsert(&test_db.pool, &release_group)
        .await
        .expect("Failed to upsert release group");

    // Retrieve and verify secondary types
    let retrieved = ReleaseGroupRepository::get_by_id(&test_db.pool, &release_group.id)
        .await
        .expect("Failed to get release group")
        .expect("Release group should exist");

    assert!(retrieved.secondary_types.is_some());
    let types = retrieved.secondary_types.unwrap();
    assert_eq!(types.len(), 2);
    assert!(types.contains(&"Compilation".to_string()));
    assert!(types.contains(&"Live".to_string()));
}

#[tokio::test]
async fn test_release_group_upsert_updates_existing() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let mut release_group = nevermind_release_group();

    // Insert release group
    ReleaseGroupRepository::upsert(&test_db.pool, &release_group)
        .await
        .expect("Failed to upsert release group");

    // Update the release group
    release_group.disambiguation = Some("25th Anniversary Edition".to_string());

    ReleaseGroupRepository::upsert(&test_db.pool, &release_group)
        .await
        .expect("Failed to upsert updated release group");

    // Verify update
    let retrieved = ReleaseGroupRepository::get_by_id(&test_db.pool, &release_group.id)
        .await
        .expect("Failed to get release group")
        .expect("Release group should exist");

    assert_eq!(
        retrieved.disambiguation,
        Some("25th Anniversary Edition".to_string())
    );
}
