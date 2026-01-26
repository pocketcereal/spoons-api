# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Removed
- Removed unused `redis` dependency from Cargo.toml
- Removed dead `cache` module (src/cache/) - Redis caching was not being used

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
