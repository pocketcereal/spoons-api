# Spoons API Reference

GraphQL API for music, podcasts, and audiobooks. Aggregates data from MusicBrainz, Audius, PodcastIndex, and LibriVox.

## Endpoints

| Path | Method | Description |
|------|--------|-------------|
| `/graphql` | POST, GET | GraphQL endpoint |
| `/graphiql` | GET | GraphiQL playground (only when auth is disabled) |
| `/healthz` | GET | Health check — returns `{ "status": "ok", "version": "..." }` |

## Authentication

All `/graphql` requests require a Bearer token when auth is enabled.

```
Authorization: Bearer <token>
```

**Auth methods** (checked in order):
1. **Supabase JWKS** — set `SPOONS_SUPABASE_URL` to your Supabase project URL
2. **JWT secret** — set `SPOONS_JWT_SECRET` for direct JWT validation
3. **Dev token** — set `SPOONS_DEV_TOKEN` (debug builds only)

Disable auth entirely with `SPOONS_AUTH_DISABLED=true`.

## ID Format

All entities use source-prefixed IDs: `<source>:<id>`

| Domain | Source | Example |
|--------|--------|---------|
| Music (Artist) | `musicbrainz` | `musicbrainz:5b11f4ce-a62d-471e-81fc-a69a8278c7da` |
| Music (Track) | `musicbrainz` | `musicbrainz:f1e4c5b8-3a6d-4e2b-9f1c-8d7e6a5b4c3d` |
| Music (Artist) | `audius` | `audius:abc123` |
| Music (Track) | `audius` | `audius:xyz789` |
| Podcast | `podcastindex` | `podcastindex:920666` |
| Episode | `podcastindex` | `podcastindex:12345` |
| Audiobook | `librivox` | `librivox:128` |
| Chapter | `librivox` | `librivox:1001` |

---

## Music Queries

### searchArtists

Search for artists across MusicBrainz and Audius.

```graphql
query {
  searchArtists(
    query: "Radiohead"
    source: MUSIC_BRAINZ  # optional: MUSIC_BRAINZ | AUDIUS
    limit: 25              # optional, default 25, max 100
    offset: 0              # optional, default 0, max 10000
  ) {
    __typename
    ... on MusicBrainzArtist {
      id
      name
      source
      sourceId
      sortName
      artistType
      disambiguation
      country
      area { id name sortName }
      lifeSpan { begin end ended }
    }
    ... on AudiusArtist {
      id
      name
      source
      sourceId
      imageUrl
      handle
      bio
      location
      isVerified
      isDeactivated
      followerCount
      followingCount
      trackCount
      playlistCount
    }
  }
}
```

When `source` is omitted, both sources are queried in parallel with a 10s timeout per source. Results are combined.

### artist

Get a single artist by ID.

```graphql
query {
  artist(id: "musicbrainz:5b11f4ce-a62d-471e-81fc-a69a8278c7da", source: MUSIC_BRAINZ) {
    ... on MusicBrainzArtist { id name country }
  }
}
```

**Arguments:**
- `id` — Source-specific ID (not prefixed). For MusicBrainz, this is a UUID.
- `source` — Required: `MUSIC_BRAINZ` or `AUDIUS`

### searchTracks

Search for tracks/recordings across MusicBrainz and Audius.

```graphql
query {
  searchTracks(
    query: "Creep"
    source: null  # optional: MUSIC_BRAINZ | AUDIUS
    limit: 25     # optional, default 25, max 100
    offset: 0     # optional, default 0, max 10000
  ) {
    __typename
    ... on MusicBrainzTrack {
      id
      title
      source
      sourceId
      durationMs
      artistName
      disambiguation
      video
    }
    ... on AudiusTrack {
      id
      title
      source
      sourceId
      durationMs
      artistName
      description
      genre
      mood
      playCount
      favoriteCount
      repostCount
      artworkUrl
      isStreamable
    }
  }
}
```

### track

Get a single track by ID.

```graphql
query {
  track(id: "f1e4c5b8-...", source: MUSIC_BRAINZ) {
    ... on MusicBrainzTrack { id title artistName }
  }
}
```

**Arguments:**
- `id` — Source-specific ID
- `source` — Required: `MUSIC_BRAINZ` or `AUDIUS`

### randomTracks

Get random tracks for discovery.

```graphql
query {
  randomTracks(
    source: null  # optional: MUSIC_BRAINZ | AUDIUS
    limit: 10     # optional, default 10, max 100
  ) {
    __typename
    ... on MusicBrainzTrack { id title artistName }
    ... on AudiusTrack { id title artistName artworkUrl }
  }
}
```

- **MusicBrainz**: Random offset into search index (capped at 10,000)
- **Audius**: Sampled from trending tracks

### randomArtists

Get random artists for discovery.

```graphql
query {
  randomArtists(
    source: null  # optional: MUSIC_BRAINZ | AUDIUS
    limit: 10     # optional, default 10, max 100
  ) {
    __typename
    ... on MusicBrainzArtist { id name country }
    ... on AudiusArtist { id name handle }
  }
}
```

- **MusicBrainz**: Random offset into search index
- **Audius**: Unique artists extracted from trending tracks

### version

Returns the API version.

```graphql
query { version }
```

---

## Podcast Queries

Requires PodcastIndex to be configured (`podcast_index.enabled: true` with API credentials).

### searchPodcasts

Search podcasts by keyword.

```graphql
query {
  searchPodcasts(
    query: "technology"
    limit: 20              # optional, default 20, max 100
    source: PODCAST_INDEX  # optional — omit to search all sources
  ) {
    ... on PodcastIndexPodcast {
      id
      title
      source
      sourceId
      author
      description
      artworkUrl
      feedUrl
      language
      categories { id name }
      episodeCount
      latestPublishTime
      itunesId
      trendScore
      podcastGuid
    }
  }
}
```

### searchPodcastsByTitle

Search podcasts by exact title match.

```graphql
query {
  searchPodcastsByTitle(title: "The Daily", limit: 10, source: PODCAST_INDEX) {
    ... on PodcastIndexPodcast { id title author }
  }
}
```

### podcast

Get a single podcast by prefixed ID.

```graphql
query {
  podcast(id: "podcastindex:920666") {
    ... on PodcastIndexPodcast { id title author episodeCount }
  }
}
```

### trendingPodcasts

Get trending podcasts, optionally filtered by category IDs.

```graphql
query {
  trendingPodcasts(
    limit: 20              # optional, default 20, max 100
    categories: [1, 2]     # optional — filter by category IDs
    source: PODCAST_INDEX  # optional — omit to query all sources
  ) {
    ... on PodcastIndexPodcast { id title trendScore }
  }
}
```

### podcastCategories

List all available podcast categories.

```graphql
query {
  podcastCategories { id name }
}
```

### episodes

Get episodes for a podcast.

```graphql
query {
  episodes(
    podcastId: "podcastindex:920666"
    limit: 20  # optional, default 20, max 100
  ) {
    ... on PodcastIndexEpisode {
      id
      title
      source
      sourceId
      podcastId
      description
      audioUrl
      durationSeconds
      publishedAt
      episodeNumber
      seasonNumber
      imageUrl
      audioType
      audioLength
      episodeType
      explicit
    }
  }
}
```

### episode

Get a single episode by prefixed ID.

```graphql
query {
  episode(id: "podcastindex:12345") {
    ... on PodcastIndexEpisode { id title audioUrl durationSeconds }
  }
}
```

### randomEpisodes

Get random episodes for discovery.

```graphql
query {
  randomEpisodes(
    limit: 10              # optional, default 10, max 100
    language: "en"          # optional — ISO 639-1 language code
    categories: [1, 2]      # optional — filter by category IDs
    source: PODCAST_INDEX   # optional — omit to query all sources
  ) {
    ... on PodcastIndexEpisode { id title podcastId audioUrl }
  }
}
```

---

## Audiobook Queries

Requires LibriVox to be configured (`librivox.enabled: true`).

### searchAudiobooks

Search audiobooks by title.

```graphql
query {
  searchAudiobooks(
    query: "pride and prejudice"
    limit: 20          # optional, default 20, max 100
    source: LIBRI_VOX  # optional — omit to search all sources
  ) {
    ... on LibriVoxAudiobook {
      id
      title
      source
      sourceId
      description
      language
      authors { firstName lastName dob dod }
      numSections
      totalTime
      totalTimeSecs
      coverartUrl
      copyrightYear
      urlTextSource
      urlZipFile
      urlLibrivox
      urlIarchive
      coverartThumbnail
    }
  }
}
```

### audiobook

Get a single audiobook by prefixed ID.

```graphql
query {
  audiobook(id: "librivox:128") {
    ... on LibriVoxAudiobook {
      id
      title
      authors { firstName lastName }
      numSections
      totalTime
    }
  }
}
```

### chapters

Get chapters for an audiobook.

```graphql
query {
  chapters(
    audiobookId: "librivox:128"
    limit: 100  # optional, default 100, max 100
  ) {
    ... on LibriVoxChapter {
      id
      title
      source
      sourceId
      audiobookId
      sectionNumber
      duration
      durationSeconds
      listenUrl
      language
      readers
    }
  }
}
```

### randomAudiobooks

Get random audiobooks for discovery.

```graphql
query {
  randomAudiobooks(
    limit: 10          # optional, default 10, max 100
    source: LIBRI_VOX  # optional — omit for all sources
  ) {
    ... on LibriVoxAudiobook { id title authors { firstName lastName } coverartUrl }
  }
}
```

Fetches a page at a random offset from the LibriVox catalog (~20,000 books), then samples. Retries up to 3 times with decreasing offset if the page is empty.

---

## Unified Queries

Cross-domain queries that fan out to music, podcasts, and audiobooks in parallel.

### search

Search across all content domains.

```graphql
query {
  search(
    query: "pride and prejudice"
    domains: [MUSIC, PODCASTS, AUDIOBOOKS]  # optional — null = all domains
    limit: 10                                # optional, default 20, per-domain limit
  ) {
    music {
      artists { ... on MusicBrainzArtist { id name } ... on AudiusArtist { id name } }
      tracks { ... on MusicBrainzTrack { id title } ... on AudiusTrack { id title } }
    }
    podcasts {
      podcasts { ... on PodcastIndexPodcast { id title author } }
    }
    audiobooks {
      audiobooks { ... on LibriVoxAudiobook { id title authors { firstName lastName } } }
    }
  }
}
```

**Arguments:**
- `query` — Search term (1–500 characters)
- `domains` — Optional array of `ContentDomain` values: `MUSIC`, `PODCASTS`, `AUDIOBOOKS`. Omit for all.
- `limit` — Per-domain result limit (default 20, max 100). Each domain gets up to this many results.

**Return type:** `SearchResults` with nullable domain fields:
- `music: { artists, tracks }` — null if not requested or failed
- `podcasts: { podcasts }` — null if not requested, not configured, or failed
- `audiobooks: { audiobooks }` — null if not requested, not configured, or failed

### random

Get random content across domains for discovery.

```graphql
query {
  random(
    domains: [AUDIOBOOKS, PODCASTS]  # optional — null = all domains
    limit: 5                          # optional, default 10, per-domain limit
  ) {
    music {
      artists { ... on MusicBrainzArtist { id name } }
      tracks { ... on MusicBrainzTrack { id title artistName } }
    }
    podcasts {
      episodes { ... on PodcastIndexEpisode { id title audioUrl } }
    }
    audiobooks {
      audiobooks { ... on LibriVoxAudiobook { id title coverartUrl } }
    }
  }
}
```

**Arguments:**
- `domains` — Optional array of `ContentDomain`. Omit for all.
- `limit` — Per-domain limit (default 10, max 100).

**Partial failure:** All domains are queried in parallel. Podcast and audiobook domains have a 10-second timeout. Music relies on per-source timeouts (10s each for MusicBrainz and Audius individually). If a domain fails or times out, its field is `null` and the failure is logged server-side. Other domains still return data.

---

## Input Validation

| Parameter | Rule |
|-----------|------|
| `query` / `title` | Trimmed, 1–500 characters |
| `id` | Trimmed, 1–64 characters |
| `limit` | Clamped to 1–100 |
| `offset` | Clamped to 0–10,000 |

## Error Codes

Errors are returned in the standard GraphQL `errors` array with an `extensions.code` field:

| Code | Meaning |
|------|---------|
| `NOT_FOUND` | Entity not found |
| `INVALID_INPUT` | Validation failed (empty query, bad ID format, etc.) |
| `FEATURE_DISABLED` | Required integration not configured |
| `INTERNAL_SERVER_ERROR` | Unexpected server error |

Example error response:

```json
{
  "data": null,
  "errors": [{
    "message": "LibriVox is not configured. Set librivox in config.yaml.",
    "extensions": { "code": "FEATURE_DISABLED" }
  }]
}
```

## Caching

All data is cached in PostgreSQL with a configurable TTL (`database.cache_ttl_seconds`, default 24 hours).

- **Single entity lookups** (`artist`, `audiobook`, `podcast`, `episode`): Cache-first — returns DB-cached data if fresh, otherwise fetches from source API and caches the result.
- **Search queries** (`searchArtists`, `searchAudiobooks`, etc.): Cache-first — caches both the search result ordering and the individual entities.
- **Random queries** (`randomTracks`, `randomAudiobooks`, etc.): Always hit the source API, but cache the individual entities returned.
- **Trending/categories**: Direct API calls (not DB-cached).

## Configuration

See `config.yaml` for all options. Key settings:

```yaml
server:
  port: 4000

database:
  max_connections: 10
  cache_ttl_seconds: 86400  # 24 hours

audius:
  enabled: true
  app_name: spoons-api

podcast_index:
  enabled: true
  api_key: ${PODCAST_INDEX_API_KEY}
  api_secret: ${PODCAST_INDEX_API_SECRET}

librivox:
  enabled: true
```

Environment variables can be interpolated with `${VAR}` syntax in the config file. The database URL is set via `DATABASE_URL` env var.
