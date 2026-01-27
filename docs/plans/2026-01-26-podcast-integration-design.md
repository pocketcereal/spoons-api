# Podcast Integration Design

**Date:** 2026-01-26
**Status:** Approved
**Scope:** Migrate podcast discovery features from pocketcereal-api to spoons-api

## Overview

Integrate PodcastIndex API into spoons-api as a new domain alongside the existing music domain. This brings podcast search, trending, categories, and episode discovery into the unified GraphQL API.

### Goals

- Add podcast discovery (search, trending, categories, details)
- Add episode management (list, details, random episodes)
- Implement server-side caching (in-memory + database)
- Follow existing multi-source architecture pattern
- Enable future podcast sources (Spotify, Apple, etc.)

### Non-Goals

- Commercial detection, transcription, annotations
- User preferences (follow/block)
- User profiles
- Job management

## Architecture

### Domain Separation

```
Music Domain (existing)
├── Artist interface → MusicBrainzArtist, AudiusArtist
└── Track interface  → MusicBrainzTrack, AudiusTrack

Podcast Domain (new)
├── Podcast interface → PodcastIndexPodcast, [future sources]
└── Episode interface → PodcastIndexEpisode, [future sources]
```

Podcasts are a distinct domain with their own types, not unified with music. This keeps domain boundaries clean while following the same architectural patterns.

### Module Structure

```
src/
├── podcast_index/              # External API client
│   ├── mod.rs
│   ├── client.rs               # PodcastIndexClient facade
│   ├── types.rs                # API response types
│   ├── auth.rs                 # HMAC signing logic
│   ├── endpoints/
│   │   ├── mod.rs
│   │   ├── search.rs
│   │   ├── trending.rs
│   │   ├── categories.rs
│   │   ├── podcasts.rs
│   │   └── episodes.rs
│   └── conversions.rs          # API types → domain types
│
├── podcast/                    # Domain layer
│   ├── mod.rs
│   ├── types.rs                # Podcast, Episode domain types
│   └── source.rs               # PodcastSource enum
│
├── graphql/
│   ├── podcast/                # Podcast GraphQL types
│   │   ├── mod.rs
│   │   ├── podcast_types.rs
│   │   ├── episode_types.rs
│   │   └── queries.rs
│   └── schema.rs               # Updated with MergedObject
│
├── db/
│   ├── models/
│   │   ├── podcast.rs
│   │   └── episode.rs
│   └── repositories/
│       ├── podcast.rs
│       └── episode.rs
│
└── cache/                      # In-memory cache
    ├── mod.rs
    ├── service.rs              # CacheService trait
    └── in_memory.rs            # InMemoryCacheService
```

## Domain Types

### Source-Agnostic Types (`src/podcast/types.rs`)

```rust
pub struct Podcast {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub artwork_url: Option<String>,
    pub feed_url: String,
    pub language: Option<String>,
    pub categories: Vec<Category>,
    pub episode_count: Option<i32>,
    pub latest_publish_time: Option<DateTime<Utc>>,
}

pub struct Episode {
    pub id: i64,
    pub podcast_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub audio_url: String,
    pub duration_seconds: Option<i32>,
    pub published_at: Option<DateTime<Utc>>,
    pub episode_number: Option<i32>,
    pub season_number: Option<i32>,
    pub image_url: Option<String>,
}

pub struct Category {
    pub id: i32,
    pub name: String,
}
```

### GraphQL Types

Interface pattern with source-specific implementations:

```rust
#[derive(Interface)]
#[graphql(field(name = "id", ty = "String"))]
#[graphql(field(name = "title", ty = "String"))]
#[graphql(field(name = "source", ty = "PodcastSource"))]
// ... other common fields
pub enum Podcast {
    PodcastIndex(PodcastIndexPodcast),
}

#[derive(SimpleObject)]
pub struct PodcastIndexPodcast {
    // Interface fields + source-specific:
    pub itunes_id: Option<i64>,
    pub trend_score: Option<i32>,
    pub podcast_guid: Option<String>,
}
```

IDs are prefixed with source: `podcastindex:12345`

## PodcastIndex API Client

### Authentication (`src/podcast_index/auth.rs`)

PodcastIndex uses HMAC-SHA1 authentication:

```rust
pub struct PodcastIndexAuth {
    api_key: String,
    api_secret: String,
}

impl PodcastIndexAuth {
    pub fn sign_request(&self) -> AuthHeaders {
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let auth_string = format!("{}{}{}", self.api_key, self.api_secret, epoch);
        let hash = sha1_hex(&auth_string);

        AuthHeaders {
            x_auth_key: self.api_key.clone(),
            x_auth_date: epoch.to_string(),
            authorization: hash,
        }
    }
}
```

### Client Facade (`src/podcast_index/client.rs`)

```rust
pub struct PodcastIndexClient {
    client: ApiClient,
    auth: PodcastIndexAuth,
}

impl PodcastIndexClient {
    pub fn new(api_key: &str, api_secret: &str) -> Result<Self>;

    pub async fn search(&self, query: &str, limit: i32) -> Result<Vec<Podcast>>;
    pub async fn trending(&self, limit: i32, categories: Option<&[i32]>) -> Result<Vec<Podcast>>;
    pub async fn categories(&self) -> Result<Vec<Category>>;
    pub async fn get_podcast(&self, feed_id: i64) -> Result<Podcast>;
    pub async fn get_episodes(&self, feed_id: i64, limit: i32) -> Result<Vec<Episode>>;
    pub async fn get_episode(&self, episode_id: i64) -> Result<Episode>;
    pub async fn random_episodes(&self, limit: i32, lang: Option<&str>) -> Result<Vec<Episode>>;
}
```

### Endpoint Modules

Each endpoint group in its own file:

- `endpoints/search.rs` - search_podcasts, search_by_title, search_by_author
- `endpoints/trending.rs` - get_trending
- `endpoints/categories.rs` - get_categories
- `endpoints/podcasts.rs` - get_podcast_by_feed_id
- `endpoints/episodes.rs` - get_episodes, get_episode, get_random_episodes

## Caching Layer

### Two-Tier Cache Strategy

1. **In-Memory Cache** - Hot data, fast access, LRU eviction
2. **Database Cache** - Persistent, survives restarts, shared across instances

### Cache Service Trait (`src/cache/service.rs`)

```rust
#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    async fn set<T: Serialize + Send>(&self, key: &str, value: &T, ttl: Duration);
    async fn remove(&self, key: &str);
    async fn clear(&self);
}
```

### In-Memory Implementation (`src/cache/in_memory.rs`)

```rust
pub struct InMemoryCacheService {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    max_entries: usize,
}

struct CacheEntry {
    data: Vec<u8>,
    expires_at: Instant,
    last_accessed: Instant,
}
```

### Cache Keys

Namespaced to avoid collisions:

```
podcast:trending:{category_ids_hash}
podcast:search:{query_hash}
podcast:detail:{feed_id}
podcast:categories
episode:list:{feed_id}:{limit}
episode:detail:{episode_id}
episode:random:{lang}:{limit}
```

### TTL Configuration

```rust
pub struct CacheConfig {
    pub enabled: bool,
    pub max_entries: usize,           // Default: 1000
    pub trending_ttl_seconds: u64,    // Default: 300 (5 min)
    pub search_ttl_seconds: u64,      // Default: 600 (10 min)
    pub podcast_ttl_seconds: u64,     // Default: 86400 (24 hr)
    pub episode_ttl_seconds: u64,     // Default: 3600 (1 hr)
    pub categories_ttl_seconds: u64,  // Default: 86400 (24 hr)
}
```

## Database Schema

### Podcasts Table

```sql
CREATE TABLE podcasts (
    id BIGINT PRIMARY KEY,              -- PodcastIndex feed_id
    title TEXT NOT NULL,
    author TEXT,
    description TEXT,
    artwork_url TEXT,
    feed_url TEXT NOT NULL,
    language TEXT,
    categories JSONB NOT NULL DEFAULT '[]',
    itunes_id BIGINT,
    episode_count INT,
    latest_publish_time TIMESTAMPTZ,
    trend_score INT,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_podcasts_title ON podcasts(title);
CREATE INDEX idx_podcasts_cached_at ON podcasts(cached_at);
```

### Episodes Table

```sql
CREATE TABLE episodes (
    id BIGINT PRIMARY KEY,              -- PodcastIndex episode_id
    podcast_id BIGINT NOT NULL REFERENCES podcasts(id),
    title TEXT NOT NULL,
    description TEXT,
    audio_url TEXT NOT NULL,
    audio_type TEXT,
    audio_length BIGINT,
    duration_seconds INT,
    published_at TIMESTAMPTZ,
    episode_number INT,
    season_number INT,
    image_url TEXT,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_episodes_podcast_id ON episodes(podcast_id);
CREATE INDEX idx_episodes_published_at ON episodes(published_at);
CREATE INDEX idx_episodes_cached_at ON episodes(cached_at);
```

### Search Cache Tables

```sql
CREATE TABLE podcast_search_cache (
    query_hash TEXT PRIMARY KEY,
    query_text TEXT NOT NULL,
    podcast_ids BIGINT[] NOT NULL,
    total_count INT NOT NULL,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE podcast_trending_cache (
    cache_key TEXT PRIMARY KEY,
    podcast_ids BIGINT[] NOT NULL,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## GraphQL Queries

### Query Root

```rust
#[derive(MergedObject, Default)]
pub struct QueryRoot(
    MusicQuery,      // Existing
    PodcastQuery,    // New
);
```

### Podcast Queries

```rust
#[Object]
impl PodcastQuery {
    async fn search_podcasts(
        &self,
        ctx: &Context<'_>,
        query: String,
        source: Option<PodcastSource>,
        #[graphql(default = 20)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> Result<Vec<Podcast>>;

    async fn trending_podcasts(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 20)] limit: i32,
        categories: Option<Vec<i32>>,
        source: Option<PodcastSource>,
    ) -> Result<Vec<Podcast>>;

    async fn podcast(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Option<Podcast>>;

    async fn podcast_categories(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<Category>>;

    async fn episodes(
        &self,
        ctx: &Context<'_>,
        podcast_id: String,
        #[graphql(default = 20)] limit: i32,
    ) -> Result<Vec<Episode>>;

    async fn episode(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Option<Episode>>;

    async fn random_episodes(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i32,
        language: Option<String>,
        categories: Option<Vec<i32>>,
    ) -> Result<Vec<Episode>>;
}
```

### Example Queries

```graphql
query SearchPodcasts {
  searchPodcasts(query: "tech news", limit: 10) {
    id
    title
    author
    artworkUrl
    ... on PodcastIndexPodcast {
      trendScore
      itunesId
    }
  }
}

query TrendingWithEpisodes {
  trendingPodcasts(limit: 5, categories: [102]) {
    id
    title
  }

  episodes(podcastId: "podcastindex:12345", limit: 3) {
    title
    durationSeconds
    publishedAt
  }
}

query RandomDiscovery {
  randomEpisodes(limit: 5, language: "en") {
    id
    title
    ... on PodcastIndexEpisode {
      podcast {
        title
        artworkUrl
      }
    }
  }
}
```

## Configuration

### Config Structs (`src/config.rs`)

```rust
#[derive(Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub database: DatabaseConfig,
    pub audius: AudiusConfig,
    pub podcast_index: PodcastIndexConfig,
    pub cache: CacheConfig,
}

#[derive(Deserialize)]
pub struct PodcastIndexConfig {
    pub enabled: bool,
    pub api_key: String,
    pub api_secret: String,
    pub base_url: Option<String>,
}
```

### Environment Variables

```bash
SPOONS_PODCAST_INDEX_ENABLED=true
SPOONS_PODCAST_INDEX_API_KEY=your_key
SPOONS_PODCAST_INDEX_API_SECRET=your_secret
SPOONS_CACHE_ENABLED=true
SPOONS_CACHE_MAX_ENTRIES=1000
```

### Config File (`config.yaml`)

```yaml
server:
  port: 4000

database:
  url: postgres://spoons:spoons@localhost:5432/spoons
  max_connections: 10

podcast_index:
  enabled: true
  api_key: ${PODCAST_INDEX_API_KEY}
  api_secret: ${PODCAST_INDEX_API_SECRET}

cache:
  enabled: true
  max_entries: 1000
  trending_ttl_seconds: 300
  search_ttl_seconds: 600
  podcast_ttl_seconds: 86400
  episode_ttl_seconds: 3600
  categories_ttl_seconds: 86400
```

---

## Implementation Plan

### Phase 1: Foundation (Parallel)

All tasks in this phase have no dependencies and can run simultaneously.

#### Task 1a: Cache Module

**Files:**
- `src/cache/mod.rs`
- `src/cache/service.rs`
- `src/cache/in_memory.rs`

**Acceptance Criteria:**
- [ ] CacheService trait defined with get/set/remove/clear
- [ ] InMemoryCacheService with TTL and LRU eviction
- [ ] NoOpCacheService for testing
- [ ] Unit tests for cache operations

---

#### Task 1b: PodcastIndex Client Skeleton + Auth

**Files:**
- `src/podcast_index/mod.rs`
- `src/podcast_index/client.rs`
- `src/podcast_index/auth.rs`
- `src/podcast_index/types.rs`

**Acceptance Criteria:**
- [ ] PodcastIndexAuth with HMAC-SHA1 signing
- [ ] PodcastIndexClient struct with new() constructor
- [ ] API response types (raw, from PodcastIndex JSON)
- [ ] Unit tests for auth signing

---

#### Task 1c: Database Migrations

**Files:**
- `migrations/YYYYMMDD_create_podcasts/up.sql`
- `migrations/YYYYMMDD_create_podcasts/down.sql`
- `src/db/schema.rs` (regenerated)

**Acceptance Criteria:**
- [ ] podcasts table with all fields
- [ ] episodes table with FK to podcasts
- [ ] podcast_search_cache table
- [ ] podcast_trending_cache table
- [ ] All indexes created
- [ ] Migrations run successfully

---

#### Task 1d: Domain Types

**Files:**
- `src/podcast/mod.rs`
- `src/podcast/types.rs`
- `src/podcast/source.rs`

**Acceptance Criteria:**
- [ ] Podcast struct (source-agnostic)
- [ ] Episode struct (source-agnostic)
- [ ] Category struct
- [ ] PodcastSource enum
- [ ] Display/FromStr for PodcastSource

---

### Phase 2: Core Implementation (Parallel)

Depends on Phase 1 completion.

#### Task 2a: PodcastIndex Endpoints

**Files:**
- `src/podcast_index/endpoints/mod.rs`
- `src/podcast_index/endpoints/search.rs`
- `src/podcast_index/endpoints/trending.rs`
- `src/podcast_index/endpoints/categories.rs`
- `src/podcast_index/endpoints/podcasts.rs`
- `src/podcast_index/endpoints/episodes.rs`
- `src/podcast_index/conversions.rs`

**Depends on:** 1b

**Acceptance Criteria:**
- [ ] search_podcasts, search_by_title, search_by_author
- [ ] get_trending with category filter
- [ ] get_categories
- [ ] get_podcast_by_feed_id
- [ ] get_episodes, get_episode, get_random_episodes
- [ ] Conversions from API types to domain types
- [ ] Client facade methods implemented

---

#### Task 2b: Database Models + Repositories

**Files:**
- `src/db/models/podcast.rs`
- `src/db/models/episode.rs`
- `src/db/models/mod.rs` (updated)
- `src/db/repositories/podcast.rs`
- `src/db/repositories/episode.rs`
- `src/db/repositories/mod.rs` (updated)

**Depends on:** 1c, 1d

**Acceptance Criteria:**
- [ ] PodcastRow, NewPodcastRow with Diesel derives
- [ ] EpisodeRow, NewEpisodeRow with Diesel derives
- [ ] PodcastRepository: get_cached, get_by_ids, upsert, upsert_many
- [ ] EpisodeRepository: get_cached, get_by_podcast_id, upsert_many
- [ ] Search cache repository methods
- [ ] Conversions between Row types and domain types

---

#### Task 2c: GraphQL Types

**Files:**
- `src/graphql/podcast/mod.rs`
- `src/graphql/podcast/podcast_types.rs`
- `src/graphql/podcast/episode_types.rs`

**Depends on:** 1d

**Acceptance Criteria:**
- [ ] Podcast interface enum
- [ ] PodcastIndexPodcast with Object impl
- [ ] Episode interface enum
- [ ] PodcastIndexEpisode with Object impl
- [ ] Category GraphQL type
- [ ] PodcastSource GraphQL enum
- [ ] From impls for domain → GraphQL types

---

#### Task 2d: Config Additions

**Files:**
- `src/config.rs` (updated)

**Depends on:** None (but logically groups with 1a, 1b)

**Acceptance Criteria:**
- [ ] PodcastIndexConfig struct
- [ ] CacheConfig struct
- [ ] AppConfig updated with new fields
- [ ] Environment variable mappings

---

### Phase 3: Integration (Parallel)

Depends on Phase 2 completion. Each query group can be implemented independently.

#### Task 3a: Search Query

**Files:**
- `src/graphql/podcast/queries.rs` (search_podcasts)

**Depends on:** 2a, 2b, 2c

**Acceptance Criteria:**
- [ ] search_podcasts resolver
- [ ] Cache check → API call → cache write pattern
- [ ] Source filter support
- [ ] Pagination (limit/offset)

---

#### Task 3b: Trending Query

**Files:**
- `src/graphql/podcast/queries.rs` (trending_podcasts)

**Depends on:** 2a, 2b, 2c

**Acceptance Criteria:**
- [ ] trending_podcasts resolver
- [ ] Category filter support
- [ ] Cache with short TTL (5 min)

---

#### Task 3c: Podcast Detail Query

**Files:**
- `src/graphql/podcast/queries.rs` (podcast)

**Depends on:** 2a, 2b, 2c

**Acceptance Criteria:**
- [ ] podcast resolver
- [ ] ID parsing (source prefix extraction)
- [ ] Cache-first pattern

---

#### Task 3d: Episode Queries

**Files:**
- `src/graphql/podcast/queries.rs` (episodes, episode, random_episodes)

**Depends on:** 2a, 2b, 2c

**Acceptance Criteria:**
- [ ] episodes resolver (by podcast ID)
- [ ] episode resolver (single by ID)
- [ ] random_episodes resolver with language/category filters

---

#### Task 3e: Categories Query

**Files:**
- `src/graphql/podcast/queries.rs` (podcast_categories)

**Depends on:** 2a, 2c

**Acceptance Criteria:**
- [ ] podcast_categories resolver
- [ ] Long TTL cache (24 hr)

---

### Phase 4: Wiring (Sequential)

Depends on all Phase 3 tasks.

#### Task 4a: Server Initialization

**Files:**
- `src/server.rs` (updated)
- `src/graphql/schema.rs` (updated)
- `src/graphql/mod.rs` (updated)

**Depends on:** All previous tasks

**Acceptance Criteria:**
- [ ] Cache service initialization
- [ ] PodcastIndex client initialization
- [ ] AppContext updated with new fields
- [ ] QueryRoot uses MergedObject with PodcastQuery
- [ ] Graceful handling when podcast_index disabled

---

### Phase 5: Testing & Polish (Parallel)

#### Task 5a: Unit Tests

**Files:**
- Tests in each module

**Acceptance Criteria:**
- [ ] Cache service tests
- [ ] Auth signing tests
- [ ] Type conversion tests
- [ ] Repository tests (with test DB)

---

#### Task 5b: Integration Tests

**Files:**
- `tests/integration/podcast_tests.rs`

**Acceptance Criteria:**
- [ ] GraphQL query tests with seeded data
- [ ] Cache behavior tests
- [ ] End-to-end flow tests

---

#### Task 5c: Smoke Tests

**Files:**
- `tests/integration/external_api_smoke_tests.rs` (updated)

**Acceptance Criteria:**
- [ ] PodcastIndex API connectivity tests
- [ ] Search, trending, categories smoke tests
- [ ] Rate limit handling verification

---

#### Task 5d: Documentation

**Files:**
- `README.md` (updated)
- `config.yaml.example` (updated)

**Acceptance Criteria:**
- [ ] Podcast queries documented
- [ ] Configuration options documented
- [ ] PodcastIndex API key setup instructions

---

## Dependency Graph

```
Phase 1 (parallel):     1a ──┬── 1b ──┬── 1c ──┬── 1d
                             │        │        │
Phase 2 (parallel):          │    2a ◄┘    2b ◄┴── 2c ── 2d
                             │     │        │       │
                             ▼     ▼        ▼       │
Phase 3 (parallel):      ┌─ 3a ── 3b ───── 3c ──── 3d ── 3e
                         │
Phase 4 (sequential):    └────────────► 4a
                                         │
Phase 5 (parallel):                  5a  5b  5c  5d
```

## Parallelization Summary

| Phase | Tasks | Max Parallel Workers |
|-------|-------|---------------------|
| 1     | 1a, 1b, 1c, 1d | 4 |
| 2     | 2a, 2b, 2c, 2d | 4 |
| 3     | 3a, 3b, 3c, 3d, 3e | 5 |
| 4     | 4a | 1 |
| 5     | 5a, 5b, 5c, 5d | 4 |

**Total Tasks:** 18
**Critical Path:** 1b → 2a → 3a → 4a (4 sequential dependencies)

## File Count Summary

| Category | New Files | Modified Files |
|----------|-----------|----------------|
| Cache module | 3 | 0 |
| PodcastIndex client | 10 | 0 |
| Domain types | 3 | 0 |
| Database | 4 | 2 |
| GraphQL | 4 | 2 |
| Config | 0 | 1 |
| Tests | 2 | 1 |
| Docs | 0 | 2 |
| **Total** | **26** | **8** |
