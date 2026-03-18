# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- Added LibriVox audiobook integration as a new content domain alongside Music and Podcasts
- Added `searchAudiobooks`, `audiobook`, `chapters`, and `randomAudiobooks` GraphQL queries
- Added cache-first `AudiobookService` with database-backed caching for audiobooks, chapters, and search results
- Added `audiobooks`, `chapters`, and `audiobook_search_cache` database tables with migrations
- Added `randomTracks` and `randomArtists` GraphQL queries with optional `source` filter and parallel fan-out across MusicBrainz and Audius
- Added Audius trending tracks endpoint (`/tracks/trending`) for random track/artist sampling
- Added MusicBrainz random selection via search offset randomization (capped at 10,000)
- Added Terraform + Docker Compose deployment for GCP single-VM setup (Caddy TLS, Redis, Supabase Postgres)
- Added unified `search` and `random` GraphQL queries for cross-domain discovery with parallel fan-out and partial failure handling
- Added `ContentDomain` enum (`MUSIC`, `PODCASTS`, `AUDIOBOOKS`) for domain filtering
- Added GET handler to GraphQL endpoint for introspection queries
- Added `db_error()` helper function to reduce database error mapping boilerplate

### Removed
- Removed unused `redis` dependency from Cargo.toml
- Removed dead `cache` module (src/cache/) - Redis caching was not being used
- Removed unused `futures` dependency from Cargo.toml

### Fixed
- Fixed `is_retryable_error` in HTTP client - removed `is_request()` check which incorrectly retried client errors (4xx)
- Improved error message when HTTP response body read fails (now includes error details instead of empty string)
- Fixed LibriVox chapters endpoint using `id` (section ID) instead of `project_id` (audiobook ID), which returned wrong data or deserialization errors

### Changed
- Refactored search_cache.rs to use new `db_error()` helper, reducing code duplication
- Extracted `MAX_BATCH_SIZE` constant and `validate_batch_size()` helper to db/helpers.rs
- Applied shared batch validation to all 4 repository files
- Applied `db_error()` helper to all repository files (27 error mappings simplified)
