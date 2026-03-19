# Spoons API

A unified GraphQL API for music, podcast, and audiobook discovery.

## Features

### Music Discovery
- Artist and track search across MusicBrainz, Audius, and Jamendo
- Source-specific filtering or cross-source fan-out
- Random artist/track discovery
- Streaming URLs from Audius and Jamendo

### Podcast Discovery
- Search podcasts by keyword or title
- Trending podcasts with category filtering
- Episode details and random episode discovery
- Powered by [PodcastIndex](https://podcastindex.org/)

### Audiobook Discovery
- Search audiobooks by title
- Chapter listings
- Random audiobook discovery
- Powered by [LibriVox](https://librivox.org/)

### Unified Queries
- `search` — cross-domain search across music, podcasts, and audiobooks in parallel
- `random` — cross-domain random discovery
- Domain filtering via `ContentDomain` enum

## Quick Start

### Prerequisites
- Rust 1.70 or later
- PostgreSQL 14 or later

### Setup

1. Clone and set up the database:
```bash
git clone <repository-url>
cd spoons-api
createdb spoons
diesel migration run
```

2. Configure environment variables:
```bash
cp .env.example .env
# Edit .env — see config.yaml for all options
```

3. Run the server:
```bash
task dev
```

The GraphQL API will be available at `http://localhost:4000/graphql`

## API Reference

See [API.md](API.md) for the full GraphQL schema, query examples, and field reference.

## Architecture

### Source Provider Pattern

Each content domain has a provider trait (`MusicProvider`, `PodcastProvider`, `AudiobookProvider`). Sources implement their domain trait and are registered in the composition root (`server.rs`). Resolvers use `fan_out_search` for parallel dispatch — they never know about specific sources.

```
MusicProvider trait
├── MusicBrainzProvider  (cached)
├── AudiusProvider
└── JamendoProvider

PodcastProvider trait
└── PodcastIndexProvider (cached)

AudiobookProvider trait
└── LibriVoxProvider     (cached)
```

Adding a new source requires implementing the trait and one line in `server.rs`.

### Caching

MusicBrainz, PodcastIndex, and LibriVox data is cached in PostgreSQL with configurable TTL. Audius and Jamendo are not DB-cached. Cache writes are fire-and-forget.

## Development

```bash
task check          # lint + unit tests
task dev            # run dev server
task test:auth      # smoke tests against running server
```

## License

[Add your license information here]
