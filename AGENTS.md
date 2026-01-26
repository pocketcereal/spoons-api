# Rust GraphQL API for indie music data

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
- MusicBrainz - open music encyclopedia
- Audius - decentralized music streaming platform

# Patterns
- Follow functional paradigms (pure functions, dependency injection)
- Follow CLEAN architecture principles
- Keep files small and use many modules. Favor more files with single purposes over cramming a lot into one file
- Module tests at least 70% coverage
- Follow industry best practices in Rust
- Always look at the official docs for library as a reference. Use the latest versions and if context7 is available use that
- Cache-first pattern: check PostgreSQL cache before external API calls
- Fire-and-forget cache updates with error logging
- GraphQL interfaces for multi-source data aggregation
