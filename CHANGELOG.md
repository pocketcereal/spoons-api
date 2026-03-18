# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- Added `randomTracks` and `randomArtists` GraphQL queries with optional `source` filter and parallel fan-out across MusicBrainz and Audius
- Added Audius trending tracks endpoint (`/tracks/trending`) for random track/artist sampling
- Added MusicBrainz random selection via search offset randomization (capped at 10,000)
- Added Terraform + Docker Compose deployment for GCP single-VM setup (Caddy TLS, Redis, Supabase Postgres)

### Removed
- Removed unused `redis` dependency from Cargo.toml
- Removed dead `cache` module (src/cache/) - Redis caching was not being used
- Removed unused `futures` dependency from Cargo.toml

### Fixed
- Fixed `is_retryable_error` in HTTP client - removed `is_request()` check which incorrectly retried client errors (4xx)
- Improved error message when HTTP response body read fails (now includes error details instead of empty string)

### Added
- Added GET handler to GraphQL endpoint for introspection queries
- Added `db_error()` helper function to reduce database error mapping boilerplate

### Changed
- Refactored search_cache.rs to use new `db_error()` helper, reducing code duplication
- Extracted `MAX_BATCH_SIZE` constant and `validate_batch_size()` helper to db/helpers.rs
- Applied shared batch validation to all 4 repository files
- Applied `db_error()` helper to all repository files (27 error mappings simplified)
