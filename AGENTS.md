# Spoons API — Unified GraphQL API for music, podcasts, and audiobooks

## Important

- All checks and tests must pass before marking an item complete. Run `task check` (lint, typecheck, tests).
- Use the Taskfile scripts for general commands like tests, running the API, and linting.
- Follow functional paradigms (pure functions, dependency injection, immutable data).
- Follow CLEAN architecture principles.
- Keep files small — favor many modules with single purposes over large files.

## External Libraries

- axum for web server
- async-graphql for GraphQL API
- clap for config loading and CLI args
- serde for serialization / deserialization
- clippy for linting
- anyhow/thiserror for error handling
- diesel + diesel-async for database (PostgreSQL)
- reqwest for HTTP client
- jsonwebtoken for JWT auth (supports Supabase JWKS)
- tracing for structured logging

## Data Sources

- MusicBrainz — open music encyclopedia (artists, tracks, releases)
- Audius — decentralized music streaming (artists, tracks with streaming)
- Jamendo — Creative Commons music (artists, tracks with direct MP3 URLs)
- PodcastIndex — podcast search, trending, episodes
- LibriVox — public domain audiobooks and chapters

## Architecture

- Source provider abstraction: `MusicProvider`, `PodcastProvider`, `AudiobookProvider` traits in `src/domain/`
- Each source implements its domain trait in `src/sources/` and delegates to its API client
- `fan_out_search` utility handles parallel dispatch with timeouts across providers
- `AppContext` holds `Vec<Arc<dyn MusicProvider>>` etc. — resolvers are source-agnostic
- Adding a new source: implement trait in `src/sources/`, register in `src/server.rs`
- Unified `search` and `random` queries fan out across all three domains in parallel
- GraphQL interfaces for multi-source data (e.g. `Artist` has `MusicBrainzArtist`, `AudiusArtist`, `JamendoArtist` variants)

## Patterns

- Keep designs minimal — start with the simplest solution using existing infrastructure
- Cache-first pattern: check PostgreSQL cache before external API calls (MusicBrainz, PodcastIndex, LibriVox)
- Fire-and-forget cache updates with error logging
- `define_search_cache!` macro generates get/cache method pairs for each entity type
- Module tests targeting at least 70% coverage
- Always reference official docs for libraries; use the latest versions

## Testing

- Use `#[test]` functions with `assert!`, `assert_eq!`, `assert_matches!`
- One concept per test, descriptive names (`test_expired_token_returns_401`)
- Arrange-Act-Assert pattern
- No raw string assertions — assert on types, status codes, or structured fields
- Run `task test` for unit tests, `task check` for the full suite

## Development Commands

- `task dev` — run the API in dev mode (port 4000)
- `task check` — lint + typecheck + tests (run before marking anything done)
- `task test` — run unit tests
- `task jwt` — get a JWT token for authenticated endpoints
- `task test:auth` — run smoke tests against running server
