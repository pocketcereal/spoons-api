//! Integration tests for recording caching.

use spoons_api::db::repositories::RecordingRepository;

use crate::common::{TestDb, paranoid_android_recording, smells_like_teen_spirit_recording};

#[tokio::test]
async fn test_recording_upsert_and_get() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let recording = smells_like_teen_spirit_recording();

    // Insert recording
    RecordingRepository::upsert(&test_db.pool, &recording)
        .await
        .expect("Failed to upsert recording");

    // Retrieve recording (with long TTL)
    let cached = RecordingRepository::get_cached(&test_db.pool, &recording.id, 86400)
        .await
        .expect("Failed to get cached recording");

    assert!(cached.is_some());
    let cached_recording = cached.unwrap();
    assert_eq!(cached_recording.id, recording.id);
    assert_eq!(cached_recording.title, recording.title);
    assert_eq!(cached_recording.length, recording.length);
    assert_eq!(cached_recording.video, recording.video);
}

#[tokio::test]
async fn test_recording_cache_expiry() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let recording = smells_like_teen_spirit_recording();

    // Insert recording
    RecordingRepository::upsert(&test_db.pool, &recording)
        .await
        .expect("Failed to upsert recording");

    // Try to get with -1 TTL (should return None as it's immediately expired)
    // We use -1 instead of 0 to avoid clock skew issues between DB and app
    let cached = RecordingRepository::get_cached(&test_db.pool, &recording.id, -1)
        .await
        .expect("Failed to get cached recording");

    assert!(
        cached.is_none(),
        "Cache should be expired with negative TTL"
    );
}

#[tokio::test]
async fn test_recording_get_by_ids() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let smells = smells_like_teen_spirit_recording();
    let paranoid = paranoid_android_recording();

    // Insert both recordings
    RecordingRepository::upsert(&test_db.pool, &smells)
        .await
        .expect("Failed to upsert Smells Like Teen Spirit");
    RecordingRepository::upsert(&test_db.pool, &paranoid)
        .await
        .expect("Failed to upsert Paranoid Android");

    // Get by IDs
    let ids: Vec<uuid::Uuid> = vec![
        uuid::Uuid::parse_str(&smells.id).unwrap(),
        uuid::Uuid::parse_str(&paranoid.id).unwrap(),
    ];

    let recordings = RecordingRepository::get_by_ids(&test_db.pool, &ids)
        .await
        .expect("Failed to get recordings by IDs");

    assert_eq!(recordings.len(), 2);

    let titles: Vec<&str> = recordings.iter().map(|r| r.title.as_str()).collect();
    assert!(titles.contains(&"Smells Like Teen Spirit"));
    assert!(titles.contains(&"Paranoid Android"));
}

#[tokio::test]
async fn test_recording_upsert_many() {
    let test_db = TestDb::new().await;
    test_db.truncate_tables().await;

    let recordings = vec![
        smells_like_teen_spirit_recording(),
        paranoid_android_recording(),
    ];

    // Upsert many
    RecordingRepository::upsert_many(&test_db.pool, &recordings)
        .await
        .expect("Failed to upsert recordings");

    // Verify both exist
    let ids: Vec<uuid::Uuid> = recordings
        .iter()
        .map(|r| uuid::Uuid::parse_str(&r.id).unwrap())
        .collect();

    let retrieved = RecordingRepository::get_by_ids(&test_db.pool, &ids)
        .await
        .expect("Failed to get recordings");

    assert_eq!(retrieved.len(), 2);
}
