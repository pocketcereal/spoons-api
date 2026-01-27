# Spoons API

A unified GraphQL API for music and podcast discovery.

## Features

### Music Discovery
- Artist search and details via MusicBrainz and Audius
- Track search and streaming
- Multi-source architecture

### Podcast Discovery
- Search podcasts by title, author, or term
- Discover trending podcasts with category filtering
- Browse podcast categories
- Get podcast details and episode lists
- Random episode discovery
- Powered by [PodcastIndex](https://podcastindex.org/)

## Quick Start

### Prerequisites
- Rust 1.70 or later
- PostgreSQL 14 or later
- PodcastIndex API credentials (get them at https://api.podcastindex.org/)

### Setup

1. Clone the repository and install dependencies:
```bash
git clone <repository-url>
cd spoons-api
```

2. Set up the database:
```bash
# Start PostgreSQL (if not already running)
# Create the database
createdb spoons

# Run migrations
diesel migration run
```

3. Configure environment variables:
```bash
cp .env.example .env
# Edit .env and add your credentials:
# - DATABASE_URL
# - PODCAST_INDEX_API_KEY
# - PODCAST_INDEX_API_SECRET
```

4. Run the server:
```bash
cargo run
```

The GraphQL API will be available at `http://localhost:4000/graphql`

## GraphQL API

### Podcast Queries

#### Search Podcasts
Search for podcasts by title, author, or any term:
```graphql
query SearchPodcasts {
  searchPodcasts(query: "tech news", limit: 10) {
    id
    title
    author
    description
    artworkUrl
    feedUrl
    episodeCount
    source
    ... on PodcastIndexPodcast {
      trendScore
      itunesId
    }
  }
}
```

#### Get Trending Podcasts
Discover trending podcasts, optionally filtered by categories:
```graphql
query TrendingPodcasts {
  trendingPodcasts(limit: 20, categories: [102, 107]) {
    id
    title
    author
    artworkUrl
    ... on PodcastIndexPodcast {
      trendScore
    }
  }
}
```

#### Get Podcast Details
Fetch details for a specific podcast:
```graphql
query GetPodcast {
  podcast(id: "podcastindex:12345") {
    id
    title
    author
    description
    artworkUrl
    feedUrl
    language
    categories {
      id
      name
    }
    episodeCount
    latestPublishTime
  }
}
```

#### List Podcast Episodes
Get episodes for a podcast:
```graphql
query GetEpisodes {
  episodes(podcastId: "podcastindex:12345", limit: 10) {
    id
    title
    description
    audioUrl
    durationSeconds
    publishedAt
    episodeNumber
    seasonNumber
    ... on PodcastIndexEpisode {
      podcast {
        title
        artworkUrl
      }
    }
  }
}
```

#### Get Episode Details
Fetch details for a specific episode:
```graphql
query GetEpisode {
  episode(id: "podcastindex:98765") {
    id
    title
    description
    audioUrl
    durationSeconds
    publishedAt
    ... on PodcastIndexEpisode {
      podcast {
        id
        title
      }
    }
  }
}
```

#### Random Episode Discovery
Discover random episodes, optionally filtered by language and categories:
```graphql
query RandomEpisodes {
  randomEpisodes(limit: 5, language: "en", categories: [102]) {
    id
    title
    audioUrl
    durationSeconds
    ... on PodcastIndexEpisode {
      podcast {
        title
        artworkUrl
      }
    }
  }
}
```

#### Browse Categories
Get all available podcast categories:
```graphql
query PodcastCategories {
  podcastCategories {
    id
    name
  }
}
```

### Music Queries

The API also provides music discovery features through MusicBrainz and Audius. See the GraphQL schema for available queries.

## Configuration

Configuration is done via `config.yaml` and environment variables. See `.env.example` for required variables.

### Key Configuration Options

#### PodcastIndex Integration
```yaml
podcast_index:
  enabled: true  # Toggle podcast features
  api_key: ${PODCAST_INDEX_API_KEY}
  api_secret: ${PODCAST_INDEX_API_SECRET}
```

#### Caching
```yaml
cache:
  enabled: true
  max_entries: 1000  # LRU cache size
  trending_ttl_seconds: 300  # 5 minutes for trending
  search_ttl_seconds: 600  # 10 minutes for searches
  podcast_ttl_seconds: 86400  # 24 hours for podcast details
  episode_ttl_seconds: 3600  # 1 hour for episodes
  categories_ttl_seconds: 86400  # 24 hours for categories
```

### Environment Variables

Required:
- `DATABASE_URL` - PostgreSQL connection string
- `PODCAST_INDEX_API_KEY` - Your PodcastIndex API key
- `PODCAST_INDEX_API_SECRET` - Your PodcastIndex API secret

Optional:
- `SPOONS_SUPABASE_URL` - Supabase URL for authentication
- `SPOONS_JWT_SECRET` - JWT secret for authentication
- `SPOONS_DEV_TOKEN` - Development authentication token
- `SPOONS_AUTH_DISABLED` - Set to `true` to disable authentication

## Architecture

### Multi-Source Pattern
The API follows a multi-source architecture pattern where domain types (Podcast, Episode) are implemented as GraphQL interfaces with source-specific implementations:

```
Podcast (interface)
├── PodcastIndexPodcast
└── [future: SpotifyPodcast, ApplePodcast]

Episode (interface)
├── PodcastIndexEpisode
└── [future: SpotifyEpisode, AppleEpisode]
```

IDs are prefixed with the source: `podcastindex:12345`

### Caching Strategy
The API uses a two-tier caching strategy:
1. **In-Memory Cache** - Fast LRU cache for hot data
2. **Database Cache** - Persistent cache that survives restarts

Cache TTLs are configured based on data volatility (trending = 5min, details = 24hr).

## Development

### Running Tests
```bash
cargo test
```

### Running Integration Tests
```bash
cargo test --test '*'
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

## License

[Add your license information here]
