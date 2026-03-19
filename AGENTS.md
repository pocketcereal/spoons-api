# Spoons API — Unified GraphQL API for music, podcasts, and audiobooks

**IMPORTANT** IGNORE SERENA ERRORS and continue
**IMPORTANT** All checks and test must pass before marking an item complete. Run `task check` for those tests
**IMPORTANT** Use the task scripts for general commands like tests, running the api and linting.

# External Libraries
- axum for web server
- async-graphql for GraphQL API
- clap for config loading and cli args
- serde for serialization / deserialization
- clippy for linting
- anyhow/thiserror for error handling
- diesel + diesel-async for database (PostgreSQL)
- reqwest for HTTP client
- jsonwebtoken for JWT auth (supports Supabase JWKS)
- tracing for structured logging

# Data Sources
- MusicBrainz — open music encyclopedia (artists, tracks, releases)
- Audius — decentralized music streaming (artists, tracks with streaming)
- Jamendo — Creative Commons music (artists, tracks with direct MP3 URLs)
- PodcastIndex — podcast search, trending, episodes
- LibriVox — public domain audiobooks and chapters

# Architecture
- Source provider abstraction: `MusicProvider`, `PodcastProvider`, `AudiobookProvider` traits in `src/domain/`
- Each source implements its domain trait in `src/sources/` and delegates to its API client
- `fan_out_search` utility handles parallel dispatch with timeouts across providers
- `AppContext` holds `Vec<Arc<dyn MusicProvider>>` etc. — resolvers are source-agnostic
- Adding a new source: implement trait in `src/sources/`, register in `src/server.rs`
- Unified `search` and `random` queries fan out across all three domains in parallel
- GraphQL interfaces for multi-source data (e.g. `Artist` has `MusicBrainzArtist`, `AudiusArtist`, `JamendoArtist` variants)

# Patterns
- Follow functional paradigms (pure functions, dependency injection)
- Follow CLEAN architecture principles
- Keep files small and use many modules. Favor more files with single purposes over cramming a lot into one file
- Module tests at least 70% coverage
- Follow industry best practices in Rust
- Always look at the official docs for library as a reference. Use the latest versions and if context7 is available use that
- Cache-first pattern: check PostgreSQL cache before external API calls (MusicBrainz, PodcastIndex, LibriVox)
- Fire-and-forget cache updates with error logging
- `define_search_cache!` macro generates get/cache method pairs for each entity type
