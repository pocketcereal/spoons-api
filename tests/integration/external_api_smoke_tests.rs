//! Smoke tests for external API datasource connectivity.
//!
//! These tests verify that the API clients can successfully connect to
//! and retrieve data from external services (MusicBrainz, Audius).
//!
//! NOTE: These tests make REAL network calls and are ignored by default.
//! Run them explicitly with:
//!   cargo test --test integration external_api_smoke_tests -- --ignored --test-threads=1
//! Or use:
//!   task test:smoke

use spoons_api::audius::AudiusClient;
use spoons_api::musicbrainz::MusicBrainzClient;
use std::time::Duration;
use tokio::time::sleep;

// Well-known stable MusicBrainz IDs
const NIRVANA_MBID: &str = "5b11f4ce-a62d-471e-81fc-a69a8278c7da";
const NEVERMIND_RELEASE_GROUP_MBID: &str = "1b022e01-4da6-387b-8658-8678046e4cef";

/// Delay to respect MusicBrainz rate limits (1 req/sec).
const RATE_LIMIT_DELAY: Duration = Duration::from_millis(1100);

// ============================================================================
// MusicBrainz API Tests
// ============================================================================

mod musicbrainz {
    use super::*;

    fn create_client() -> MusicBrainzClient {
        MusicBrainzClient::default_client().expect("Failed to create MusicBrainz client")
    }

    // --- Artist Tests ---

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_search_artists() {
        sleep(RATE_LIMIT_DELAY).await;
        let client = create_client();
        let results = client
            .search_artists("Nirvana", 5, 0)
            .await
            .expect("Failed to search artists");

        assert!(!results.is_empty(), "Should return at least one artist");

        let first = &results[0];
        assert!(!first.id.is_empty(), "Artist should have an ID");
        assert!(!first.name.is_empty(), "Artist should have a name");
    }

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_get_artist() {
        sleep(RATE_LIMIT_DELAY).await;
        let client = create_client();
        let artist = client
            .get_artist(NIRVANA_MBID)
            .await
            .expect("Failed to get artist by ID");

        assert_eq!(artist.id, NIRVANA_MBID);
        assert_eq!(artist.name, "Nirvana");
    }

    // --- Release Tests ---

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_search_releases() {
        sleep(RATE_LIMIT_DELAY).await;
        let client = create_client();
        let results = client
            .search_releases("Nevermind", 5, 0)
            .await
            .expect("Failed to search releases");

        assert!(!results.is_empty(), "Should return at least one release");

        let first = &results[0];
        assert!(!first.id.is_empty(), "Release should have an ID");
        assert!(!first.title.is_empty(), "Release should have a title");
    }

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_get_release_via_search() {
        sleep(RATE_LIMIT_DELAY).await;
        let client = create_client();

        // Search first to get a valid release ID
        let results = client
            .search_releases("Nevermind Nirvana", 1, 0)
            .await
            .expect("Failed to search releases");

        assert!(!results.is_empty(), "Should find at least one release");

        let release_id = &results[0].id;

        // Now fetch by ID
        let release = client
            .get_release(release_id)
            .await
            .expect("Failed to get release by ID");

        assert_eq!(&release.id, release_id);
        assert!(!release.title.is_empty(), "Release should have a title");
    }

    // --- Recording Tests ---

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_search_recordings() {
        sleep(RATE_LIMIT_DELAY).await;
        let client = create_client();
        let results = client
            .search_recordings("Smells Like Teen Spirit", 5, 0)
            .await
            .expect("Failed to search recordings");

        assert!(!results.is_empty(), "Should return at least one recording");

        let first = &results[0];
        assert!(!first.id.is_empty(), "Recording should have an ID");
        assert!(!first.title.is_empty(), "Recording should have a title");
    }

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_get_recording_via_search() {
        sleep(RATE_LIMIT_DELAY).await;
        let client = create_client();

        // Search first to get a valid recording ID
        let results = client
            .search_recordings("Smells Like Teen Spirit Nirvana", 1, 0)
            .await
            .expect("Failed to search recordings");

        assert!(!results.is_empty(), "Should find at least one recording");

        let recording_id = &results[0].id;

        // Now fetch by ID
        let recording = client
            .get_recording(recording_id)
            .await
            .expect("Failed to get recording by ID");

        assert_eq!(&recording.id, recording_id);
        assert!(!recording.title.is_empty(), "Recording should have a title");
    }

    // --- Release Group Tests ---

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_search_release_groups() {
        sleep(RATE_LIMIT_DELAY).await;
        let client = create_client();
        let results = client
            .search_release_groups("Nevermind Nirvana", 5, 0)
            .await
            .expect("Failed to search release groups");

        assert!(
            !results.is_empty(),
            "Should return at least one release group"
        );

        let first = &results[0];
        assert!(!first.id.is_empty(), "Release group should have an ID");
        assert!(!first.title.is_empty(), "Release group should have a title");
    }

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_get_release_group() {
        sleep(RATE_LIMIT_DELAY).await;
        let client = create_client();
        let release_group = client
            .get_release_group(NEVERMIND_RELEASE_GROUP_MBID)
            .await
            .expect("Failed to get release group by ID");

        assert_eq!(release_group.id, NEVERMIND_RELEASE_GROUP_MBID);
        assert!(
            release_group.title.to_lowercase().contains("nevermind"),
            "Release group title should contain 'Nevermind', got: {}",
            release_group.title
        );
    }
}

// ============================================================================
// Audius API Tests
// ============================================================================

mod audius {
    use super::*;

    async fn create_client() -> AudiusClient {
        AudiusClient::new("spoons-api-smoke-test")
            .await
            .expect("Failed to create Audius client")
    }

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_host_discovery() {
        let client = create_client().await;
        let hosts = client.hosts();

        assert!(
            !hosts.is_empty(),
            "Should discover at least one Audius host"
        );
    }

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_search_users() {
        let client = create_client().await;
        let results = client
            .search_users("deadmau5", 5, 0)
            .await
            .expect("Failed to search users");

        assert!(!results.is_empty(), "Should return at least one user");

        let first = &results[0];
        assert!(!first.id.is_empty(), "User should have an ID");
        assert!(!first.name.is_empty(), "User should have a name");
    }

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_search_tracks() {
        let client = create_client().await;
        let results = client
            .search_tracks("electronic", 5, 0)
            .await
            .expect("Failed to search tracks");

        assert!(!results.is_empty(), "Should return at least one track");

        let first = &results[0];
        assert!(!first.id.is_empty(), "Track should have an ID");
        assert!(!first.title.is_empty(), "Track should have a title");
    }

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_get_user_via_search() {
        let client = create_client().await;

        // Search first to get a valid user ID
        let results = client
            .search_users("deadmau5", 1, 0)
            .await
            .expect("Failed to search users");

        assert!(!results.is_empty(), "Should find at least one user");

        let user_id = &results[0].id;

        // Now fetch by ID
        let user = client
            .get_user(user_id)
            .await
            .expect("Failed to get user by ID");

        assert_eq!(&user.id, user_id);
        assert!(!user.name.is_empty(), "User should have a name");
    }

    #[tokio::test]
    #[ignore = "Smoke test - run explicitly with --ignored"]
    async fn smoke_test_get_track_via_search() {
        let client = create_client().await;

        // Search first to get a valid track ID
        let results = client
            .search_tracks("electronic music", 1, 0)
            .await
            .expect("Failed to search tracks");

        assert!(!results.is_empty(), "Should find at least one track");

        let track_id = &results[0].id;

        // Now fetch by ID
        let track = client
            .get_track(track_id)
            .await
            .expect("Failed to get track by ID");

        assert_eq!(&track.id, track_id);
        assert!(!track.title.is_empty(), "Track should have a title");
    }
}
