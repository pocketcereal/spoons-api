# Source Provider Abstraction + Jamendo Integration

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace hardcoded per-source wiring with a trait-based source provider system, then add Jamendo as the first new source through the clean interface.

**Architecture:** Three domain traits (`MusicProvider`, `PodcastProvider`, `AudiobookProvider`) define the contract. Each external API implements its domain trait. A generic `fan_out_search` utility handles parallel dispatch with timeouts. `AppContext` holds `Vec<Arc<dyn MusicProvider>>` etc. Resolvers become source-agnostic. Caching stays internal to each source implementation (wrapping the existing service layer).

**Tech Stack:** Rust 2024 edition (native async traits), async-graphql, tokio, reqwest, diesel-async, PostgreSQL.

**Key design decisions:**
- Trait names use `Provider` suffix to avoid collision with existing source enums (`PodcastSource`, `AudiobookSource`)
- GraphQL types keep the existing interface enum pattern (`Artist::MusicBrainz(...)`, `Artist::Audius(...)`, etc.). Adding a new source requires adding a variant + struct with `#[Object]` impl satisfying interface fields. This is a small, localized change per source.
- Existing client modules (`src/musicbrainz/`, `src/audius/`, etc.) stay in place. Thin wrapper structs in `src/sources/` implement the traits and delegate to them.
- Each source manages its own caching internally (using existing `services/` and `db/` infrastructure). The JSONB generic cache from the design spec is deferred — it can be added when per-entity tables become unwieldy. Jamendo launches without DB caching (API responses are fast enough for a prototype).

**Naming conventions:**
- `MusicProvider` trait (not `MusicSource` — avoids collision)
- `MusicBrainzProvider` struct (implements `MusicProvider`)
- Existing `DataSource` enum gets `Jamendo` variant
- Existing `PodcastSource`, `AudiobookSource` enums unchanged

**Deviation from design spec:**
- Design spec proposes flat domain structs replacing GraphQL interface enums. We keep interface enums — they're already working and clients may depend on the typed fragments. Flat structs can be a future migration.
- Design spec proposes `CachedMusicSource` decorator + JSONB tables. We defer this — each provider wraps its own service layer for caching. Jamendo has no caching initially.

---

## Task 1: Define Domain Traits

**Files:**
- Create: `src/domain/music.rs`
- Create: `src/domain/podcast.rs`
- Create: `src/domain/audiobook.rs`
- Modify: `src/domain/mod.rs`
- Modify: `src/domain/source.rs`

This task defines the three provider traits and extends the `DataSource` enum. No behavior changes — just new types.

- [ ] **Step 1: Add `Jamendo` variant to `DataSource`**

In `src/domain/source.rs`, add `Jamendo` to the enum and update `Display`, `as_str()`, `format_id()`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Enum)]
pub enum DataSource {
    MusicBrainz,
    Audius,
    Jamendo,
}
```

Update the `Display` impl, `as_str()`, and `format_id()` to handle `Jamendo => "jamendo"`.

- [ ] **Step 2: Define `MusicProvider` trait**

Create `src/domain/music.rs`:

```rust
use crate::domain::DataSource;
use crate::error::Result;
use crate::graphql::types::{Artist, Track};

/// Trait for music content providers (MusicBrainz, Audius, Jamendo, etc.)
///
/// Each provider returns GraphQL interface enum types. The provider is responsible
/// for converting its API-specific types into the correct enum variant.
pub trait MusicProvider: Send + Sync {
    fn source_id(&self) -> DataSource;
    async fn search_artists(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Artist>>;
    async fn search_tracks(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Track>>;
    async fn get_artist(&self, id: &str) -> Result<Option<Artist>>;
    async fn get_track(&self, id: &str) -> Result<Option<Track>>;
    async fn random_artists(&self, limit: i32) -> Result<Vec<Artist>>;
    async fn random_tracks(&self, limit: i32) -> Result<Vec<Track>>;
}
```

Note: `get_artist` and `get_track` return `Result<Option<T>>`. The existing `MusicService::get_artist()` returns `Result<Artist>` (not Optional). Provider impls must wrap: `Ok(Some(self.service.get_artist(id).await?))` and handle `NotFound` errors by converting to `Ok(None)`.

- [ ] **Step 3: Define `PodcastProvider` trait**

Create `src/domain/podcast.rs`:

```rust
use crate::error::Result;
use crate::graphql::podcast::{Category, Episode, Podcast};
use crate::podcast::PodcastSource;

pub trait PodcastProvider: Send + Sync {
    fn source_id(&self) -> PodcastSource;
    async fn search_podcasts(&self, query: &str, limit: i32) -> Result<Vec<Podcast>>;
    async fn search_by_title(&self, title: &str, limit: i32) -> Result<Vec<Podcast>>;
    async fn get_podcast(&self, id: i64) -> Result<Option<Podcast>>;
    async fn get_episodes(&self, podcast_id: i64, limit: i32) -> Result<Vec<Episode>>;
    async fn get_episode(&self, id: i64) -> Result<Option<Episode>>;
    async fn trending(&self, limit: i32, categories: Option<&[i32]>) -> Result<Vec<Podcast>>;
    async fn categories(&self) -> Result<Vec<Category>>;
    async fn random_episodes(&self, limit: i32, language: Option<&str>, categories: Option<&[i32]>) -> Result<Vec<Episode>>;
}
```

Note: The `get_podcast` / `get_episode` methods take `i64` because callers parse prefixed string IDs via `PodcastSource::parse_id()` before calling the provider. The resolver remains responsible for ID parsing and routing to the correct provider.

- [ ] **Step 4: Define `AudiobookProvider` trait**

Create `src/domain/audiobook.rs`:

```rust
use crate::audiobook::AudiobookSource;
use crate::error::Result;
use crate::graphql::audiobook::{Audiobook, Chapter};

pub trait AudiobookProvider: Send + Sync {
    fn source_id(&self) -> AudiobookSource;
    async fn search_audiobooks(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Audiobook>>;
    async fn get_audiobook(&self, id: i64) -> Result<Option<Audiobook>>;
    async fn get_chapters(&self, audiobook_id: i64, limit: i32) -> Result<Vec<Chapter>>;
    async fn random_audiobooks(&self, limit: i32) -> Result<Vec<Audiobook>>;
}
```

Same ID parsing note as PodcastProvider.

- [ ] **Step 5: Update `src/domain/mod.rs`**

```rust
mod audiobook;
mod music;
mod podcast;
mod source;

pub use audiobook::AudiobookProvider;
pub use music::MusicProvider;
pub use podcast::PodcastProvider;
pub use source::DataSource;
```

- [ ] **Step 6: Verify it compiles**

Run: `cargo check`
Expected: PASS (traits defined but not yet implemented)

- [ ] **Step 7: Commit**

```
[Abstraction]: Define MusicProvider, PodcastProvider, AudiobookProvider traits
```

---

## Task 2: Fan-Out Search Utility

**Files:**
- Create: `src/sources/mod.rs`
- Create: `src/sources/fan_out.rs`
- Modify: `src/lib.rs`
- Modify: `Cargo.toml` (if `futures` not present)

The generic fan-out function replaces the hardcoded `search_sources()` in `schema.rs`. It works with any slice of trait objects.

- [ ] **Step 1: Add `futures` dependency if not present**

Run: `cargo add futures`

- [ ] **Step 2: Implement `fan_out_search` with tests**

Create `src/sources/fan_out.rs`:

```rust
use std::sync::Arc;
use std::time::Duration;
use futures::future::join_all;
use crate::error::Result;

pub const SOURCE_TIMEOUT: Duration = Duration::from_secs(10);

/// Dispatches a search to multiple sources in parallel, combining results.
/// Failures and timeouts are logged but don't fail the overall query.
pub async fn fan_out_search<S, T, F, Fut>(
    sources: &[Arc<S>],
    timeout_duration: Duration,
    search_fn: F,
) -> Vec<T>
where
    S: ?Sized + Send + Sync,
    T: Send + 'static,
    F: Fn(Arc<S>) -> Fut + Send,
    Fut: std::future::Future<Output = Result<Vec<T>>> + Send,
{
    let futures = sources.iter().map(|s| {
        let source = Arc::clone(s);
        let fut = search_fn(source);
        async move {
            match tokio::time::timeout(timeout_duration, fut).await {
                Ok(Ok(items)) => items,
                Ok(Err(e)) => {
                    tracing::warn!(error = %e, "Source query failed");
                    vec![]
                }
                Err(_) => {
                    tracing::warn!("Source query timed out");
                    vec![]
                }
            }
        }
    });

    join_all(futures).await.into_iter().flatten().collect()
}

/// Dispatches a single-entity lookup to multiple sources, returning the first Some result.
pub async fn fan_out_single<S, T, F, Fut>(
    sources: &[Arc<S>],
    timeout_duration: Duration,
    search_fn: F,
) -> Result<Option<T>>
where
    S: ?Sized + Send + Sync,
    T: Send + 'static,
    F: Fn(Arc<S>) -> Fut + Send,
    Fut: std::future::Future<Output = Result<Option<T>>> + Send,
{
    let futures = sources.iter().map(|s| {
        let source = Arc::clone(s);
        let fut = search_fn(source);
        tokio::time::timeout(timeout_duration, fut)
    });

    for result in join_all(futures).await {
        match result {
            Ok(Ok(Some(item))) => return Ok(Some(item)),
            Ok(Ok(None)) => continue,
            Ok(Err(e)) => tracing::warn!(error = %e, "Source lookup failed"),
            Err(_) => tracing::warn!("Source lookup timed out"),
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    #[tokio::test]
    async fn test_fan_out_combines_results() {
        let sources: Vec<Arc<dyn Fn() -> Result<Vec<i32>> + Send + Sync>> = vec![
            Arc::new(|| Ok(vec![1, 2])),
            Arc::new(|| Ok(vec![3, 4])),
        ];
        // Test that results from multiple sources are combined
        // (Use a simple closure-based mock, or define a test trait impl)
    }

    #[tokio::test]
    async fn test_fan_out_logs_failures_without_failing() {
        // One source returns Err, the other returns Ok
        // Result should contain items from the successful source only
    }

    #[tokio::test]
    async fn test_fan_out_handles_timeout() {
        // One source sleeps longer than timeout
        // Result should contain items from the non-sleeping source
    }

    #[tokio::test]
    async fn test_fan_out_single_returns_first_some() {
        // First source returns None, second returns Some(x)
        // Result should be Some(x)
    }
}
```

Note: Tests need a concrete type to work with `fan_out_search`. Define a minimal test trait or use a wrapper struct. The exact test implementation is left to the implementer — the key behaviors to test are listed above.

- [ ] **Step 3: Wire module**

Create `src/sources/mod.rs`:
```rust
mod fan_out;
pub use fan_out::{fan_out_search, fan_out_single, SOURCE_TIMEOUT};
```

Add `pub mod sources;` to `src/lib.rs`.

- [ ] **Step 4: Run tests**

Run: `cargo test --lib -- sources::fan_out`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```
[Abstraction]: Add generic fan_out_search and fan_out_single utilities
```

---

## Task 3: Implement Source Providers (MusicBrainz, Audius, PodcastIndex, LibriVox)

**Files:**
- Create: `src/sources/musicbrainz.rs`
- Create: `src/sources/audius.rs`
- Create: `src/sources/podcast_index.rs`
- Create: `src/sources/librivox.rs`
- Modify: `src/sources/mod.rs`

All four providers are created but NOT yet wired into AppContext — the existing code continues to work. These are new code only, no modifications to existing behavior.

- [ ] **Step 1: Implement MusicBrainzProvider**

Create `src/sources/musicbrainz.rs`:

```rust
use crate::domain::{DataSource, MusicProvider};
use crate::error::{AppError, Result};
use crate::graphql::helpers::random_sample;
use crate::graphql::types::{Artist, Track};
use crate::services::MusicService;

const MUSICBRAINZ_MAX_OFFSET: i64 = 10_000;

pub struct MusicBrainzProvider {
    service: MusicService,
}

impl MusicBrainzProvider {
    pub fn new(service: MusicService) -> Self {
        Self { service }
    }
}

impl MusicProvider for MusicBrainzProvider {
    fn source_id(&self) -> DataSource { DataSource::MusicBrainz }

    async fn search_artists(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Artist>> {
        let results = self.service.search_artists(query, limit, offset).await?;
        Ok(results.into_iter().map(|a| Artist::MusicBrainz(a.into())).collect())
    }

    async fn search_tracks(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Track>> {
        let results = self.service.search_recordings(query, limit, offset).await?;
        Ok(results.into_iter().map(|r| Track::MusicBrainz(r.into())).collect())
    }

    async fn get_artist(&self, id: &str) -> Result<Option<Artist>> {
        // MusicService::get_artist returns Result<Artist>, not Option.
        // Convert NotFound to None.
        match self.service.get_artist(id).await {
            Ok(a) => Ok(Some(Artist::MusicBrainz(a.into()))),
            Err(AppError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_track(&self, id: &str) -> Result<Option<Track>> {
        match self.service.get_recording(id).await {
            Ok(r) => Ok(Some(Track::MusicBrainz(r.into()))),
            Err(AppError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn random_artists(&self, limit: i32) -> Result<Vec<Artist>> {
        // Moved from schema.rs::random_musicbrainz_artists
        let (count, _) = self.service.mb_client()
            .search_artists_with_count("*", 1, 0).await?;
        if count == 0 { return Ok(vec![]); }

        let max_offset = count.min(MUSICBRAINZ_MAX_OFFSET) - 1;
        let offset = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=max_offset as i32);

        let artists = self.service.mb_client()
            .search_artists("*", limit, offset).await?;
        Ok(random_sample(artists, limit as usize)
            .into_iter().map(|a| Artist::MusicBrainz(a.into())).collect())
    }

    async fn random_tracks(&self, limit: i32) -> Result<Vec<Track>> {
        // Moved from schema.rs::random_musicbrainz_tracks
        let (count, _) = self.service.mb_client()
            .search_recordings_with_count("*", 1, 0).await?;
        if count == 0 { return Ok(vec![]); }

        let max_offset = count.min(MUSICBRAINZ_MAX_OFFSET) - 1;
        let offset = rand::Rng::gen_range(&mut rand::thread_rng(), 0..=max_offset as i32);

        let recordings = self.service.mb_client()
            .search_recordings("*", limit, offset).await?;
        Ok(random_sample(recordings, limit as usize)
            .into_iter().map(|r| Track::MusicBrainz(r.into())).collect())
    }
}
```

- [ ] **Step 2: Implement AudiusProvider**

Create `src/sources/audius.rs`:

```rust
use crate::audius::AudiusClient;
use crate::domain::{DataSource, MusicProvider};
use crate::error::Result;
use crate::graphql::helpers::random_sample;
use crate::graphql::types::{Artist, Track};

pub struct AudiusProvider {
    client: AudiusClient,
}

impl AudiusProvider {
    pub fn new(client: AudiusClient) -> Self {
        Self { client }
    }
}

impl MusicProvider for AudiusProvider {
    fn source_id(&self) -> DataSource { DataSource::Audius }

    async fn search_artists(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Artist>> {
        let users = self.client.search_users(query, limit, offset).await?;
        Ok(users.into_iter().map(|u| Artist::Audius(u.into())).collect())
    }

    async fn search_tracks(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Track>> {
        let tracks = self.client.search_tracks(query, limit, offset).await?;
        Ok(tracks.into_iter().map(|t| Track::Audius(t.into())).collect())
    }

    async fn get_artist(&self, id: &str) -> Result<Option<Artist>> {
        let user = self.client.get_user(id).await?;
        Ok(Some(Artist::Audius(user.into())))
    }

    async fn get_track(&self, id: &str) -> Result<Option<Track>> {
        let track = self.client.get_track(id).await?;
        Ok(Some(Track::Audius(track.into())))
    }

    async fn random_artists(&self, limit: i32) -> Result<Vec<Artist>> {
        // Moved from schema.rs::random_audius_artists
        // Fetch trending, extract unique artists, sample
        let tracks = self.client.trending_tracks(limit * 3).await?;
        let mut seen = std::collections::HashSet::new();
        let artists: Vec<Artist> = tracks.into_iter()
            .filter_map(|t| {
                let user = t.user?;
                if seen.insert(user.id.clone()) { Some(Artist::Audius(user.into())) }
                else { None }
            })
            .collect();
        Ok(random_sample(artists, limit as usize))
    }

    async fn random_tracks(&self, limit: i32) -> Result<Vec<Track>> {
        // Moved from schema.rs::random_audius_tracks
        let tracks = self.client.trending_tracks(limit * 2).await?;
        let tracks: Vec<Track> = tracks.into_iter()
            .map(|t| Track::Audius(t.into()))
            .collect();
        Ok(random_sample(tracks, limit as usize))
    }
}
```

- [ ] **Step 3: Implement PodcastIndexProvider**

Create `src/sources/podcast_index.rs`:

```rust
use crate::domain::PodcastProvider;
use crate::error::Result;
use crate::graphql::podcast::{Category, Episode, Podcast};
use crate::podcast::PodcastSource;
use crate::services::PodcastService;

pub struct PodcastIndexProvider {
    service: PodcastService,
}

impl PodcastIndexProvider {
    pub fn new(service: PodcastService) -> Self {
        Self { service }
    }
}

impl PodcastProvider for PodcastIndexProvider {
    fn source_id(&self) -> PodcastSource { PodcastSource::PodcastIndex }

    async fn search_podcasts(&self, query: &str, limit: i32) -> Result<Vec<Podcast>> {
        let results = self.service.search_podcasts(query, limit).await?;
        Ok(results.into_iter().map(Podcast::from).collect())
    }

    async fn search_by_title(&self, title: &str, limit: i32) -> Result<Vec<Podcast>> {
        let results = self.service.search_by_title(title, limit).await?;
        Ok(results.into_iter().map(Podcast::from).collect())
    }

    async fn get_podcast(&self, id: i64) -> Result<Option<Podcast>> {
        let result = self.service.get_podcast(id).await?;
        Ok(result.map(Podcast::from))
    }

    async fn get_episodes(&self, podcast_id: i64, limit: i32) -> Result<Vec<Episode>> {
        let results = self.service.get_episodes(podcast_id, limit).await?;
        Ok(results.into_iter().map(Episode::from).collect())
    }

    async fn get_episode(&self, id: i64) -> Result<Option<Episode>> {
        let result = self.service.get_episode(id).await?;
        Ok(result.map(Episode::from))
    }

    // trending, categories, random_episodes go through service.client() directly
    // because PodcastService doesn't wrap these — the resolver called client() directly
    async fn trending(&self, limit: i32, categories: Option<&[i32]>) -> Result<Vec<Podcast>> {
        let results = self.service.client().trending(limit, categories).await?;
        Ok(results.into_iter().map(Podcast::from).collect())
    }

    async fn categories(&self) -> Result<Vec<Category>> {
        let results = self.service.client().categories().await?;
        Ok(results.into_iter().map(Category::from).collect())
    }

    async fn random_episodes(&self, limit: i32, language: Option<&str>, categories: Option<&[i32]>) -> Result<Vec<Episode>> {
        let results = self.service.client().random_episodes(limit, language, categories).await?;
        Ok(results.into_iter().map(Episode::from).collect())
    }
}
```

- [ ] **Step 4: Implement LibriVoxProvider**

Create `src/sources/librivox.rs`:

```rust
use crate::audiobook::AudiobookSource;
use crate::domain::AudiobookProvider;
use crate::error::Result;
use crate::graphql::audiobook::{Audiobook, Chapter};
use crate::graphql::helpers::random_sample;
use crate::services::AudiobookService;

const LIBRIVOX_MAX_OFFSET: i64 = 20_000;
const RANDOM_RETRY_ATTEMPTS: u32 = 3;

pub struct LibriVoxProvider {
    service: AudiobookService,
}

impl LibriVoxProvider {
    pub fn new(service: AudiobookService) -> Self {
        Self { service }
    }
}

impl AudiobookProvider for LibriVoxProvider {
    fn source_id(&self) -> AudiobookSource { AudiobookSource::LibriVox }

    async fn search_audiobooks(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Audiobook>> {
        let results = self.service.search_audiobooks(query, limit, offset).await?;
        Ok(results.into_iter().map(Audiobook::from).collect())
    }

    async fn get_audiobook(&self, id: i64) -> Result<Option<Audiobook>> {
        let result = self.service.get_audiobook(id).await?;
        Ok(result.map(Audiobook::from))
    }

    async fn get_chapters(&self, audiobook_id: i64, limit: i32) -> Result<Vec<Chapter>> {
        let results = self.service.get_chapters(audiobook_id, limit).await?;
        Ok(results.into_iter().map(Chapter::from).collect())
    }

    async fn random_audiobooks(&self, limit: i32) -> Result<Vec<Audiobook>> {
        // Moved from graphql/audiobook/queries.rs::random_librivox_audiobooks
        let fetch_limit = limit * 2;
        let mut offset = rand::Rng::gen_range(
            &mut rand::thread_rng(), 0..LIBRIVOX_MAX_OFFSET as i32
        );

        for _ in 0..RANDOM_RETRY_ATTEMPTS {
            let results = self.service.get_audiobooks_page(fetch_limit, offset).await?;
            if !results.is_empty() {
                return Ok(random_sample(results, limit as usize)
                    .into_iter().map(Audiobook::from).collect());
            }
            offset /= 2;
        }
        Ok(Vec::new())
    }
}
```

- [ ] **Step 5: Export all providers in sources/mod.rs**

```rust
mod audius;
mod fan_out;
mod librivox;
mod musicbrainz;
mod podcast_index;

pub use audius::AudiusProvider;
pub use fan_out::{fan_out_search, fan_out_single, SOURCE_TIMEOUT};
pub use librivox::LibriVoxProvider;
pub use musicbrainz::MusicBrainzProvider;
pub use podcast_index::PodcastIndexProvider;
```

- [ ] **Step 6: Verify it compiles**

Run: `cargo check`
Expected: PASS — providers exist but are unused (will get warnings, that's fine).

- [ ] **Step 7: Commit**

```
[Abstraction]: Implement all four source providers (MusicBrainz, Audius, PodcastIndex, LibriVox)
```

---

## Task 4: Rewire AppContext, Server, and All Resolvers (Atomic)

**Files:**
- Modify: `src/graphql/schema.rs` — AppContext, MusicQuery, helper removal
- Modify: `src/graphql/mod.rs` — update re-exports
- Modify: `src/graphql/podcast/queries.rs` — use providers
- Modify: `src/graphql/audiobook/queries.rs` — use providers
- Modify: `src/graphql/unified/queries.rs` — use providers
- Modify: `src/server.rs` — build providers

**IMPORTANT:** This task is atomic. AppContext field changes break all resolvers simultaneously, so all must be updated in one commit. Do NOT commit intermediate non-compiling states.

- [ ] **Step 1: Update AppContext in `src/graphql/schema.rs`**

Replace:
```rust
pub struct AppContext {
    pub music: MusicService,
    pub podcast: Option<PodcastService>,
    pub audiobook: Option<AudiobookService>,
}
```

With:
```rust
use crate::domain::{AudiobookProvider, MusicProvider, PodcastProvider};

#[derive(Clone)]
pub struct AppContext {
    pub music_providers: Vec<Arc<dyn MusicProvider>>,
    pub podcast_providers: Vec<Arc<dyn PodcastProvider>>,
    pub audiobook_providers: Vec<Arc<dyn AudiobookProvider>>,
}
```

- [ ] **Step 2: Update `server.rs` to build providers**

```rust
use crate::sources::{
    AudiusProvider, LibriVoxProvider, MusicBrainzProvider, PodcastIndexProvider,
};

// ... after building services as before ...

let mut music_providers: Vec<Arc<dyn MusicProvider>> = vec![
    Arc::new(MusicBrainzProvider::new(music_service)),
];
if let Some(audius) = audius_client {
    music_providers.push(Arc::new(AudiusProvider::new(audius)));
}

let mut podcast_providers: Vec<Arc<dyn PodcastProvider>> = vec![];
if let Some(ps) = podcast_service {
    podcast_providers.push(Arc::new(PodcastIndexProvider::new(ps)));
}

let mut audiobook_providers: Vec<Arc<dyn AudiobookProvider>> = vec![];
if let Some(abs) = audiobook_service {
    audiobook_providers.push(Arc::new(LibriVoxProvider::new(abs)));
}

let app_context = AppContext {
    music_providers,
    podcast_providers,
    audiobook_providers,
};
```

- [ ] **Step 3: Refactor MusicQuery in `schema.rs`**

Replace all `search_artists`, `search_tracks`, `random_artists`, `random_tracks` to use `fan_out_search`:

```rust
use crate::sources::{fan_out_search, SOURCE_TIMEOUT};

fn filter_music_providers(
    providers: &[Arc<dyn MusicProvider>],
    source: Option<DataSource>,
) -> Vec<Arc<dyn MusicProvider>> {
    match source {
        Some(s) => providers.iter().filter(|p| p.source_id() == s).cloned().collect(),
        None => providers.to_vec(),
    }
}

// In MusicQuery impl:
async fn search_artists(&self, ctx: &Context<'_>, query: String,
    source: Option<DataSource>, #[graphql(default = 25)] limit: i32,
    #[graphql(default = 0)] offset: i32,
) -> Result<Vec<Artist>> {
    let query = validate_query(&query)?;
    let limit = clamp_limit(limit);
    let app_ctx = get_app_context(ctx)?;
    let providers = filter_music_providers(&app_ctx.music_providers, source);
    Ok(fan_out_search(&providers, SOURCE_TIMEOUT, |p| {
        let q = query.clone();
        async move { p.search_artists(&q, limit, offset).await }
    }).await)
}
```

For single-entity lookups (`artist`, `track`), find the matching provider by source and call directly:

```rust
async fn artist(&self, ctx: &Context<'_>, id: String, source: DataSource) -> Result<Option<Artist>> {
    let id = validate_id(&id)?;
    let app_ctx = get_app_context(ctx)?;
    let provider = app_ctx.music_providers.iter()
        .find(|p| p.source_id() == source)
        .ok_or_else(|| AppError::InvalidInput(format!("Unknown music source: {:?}", source)).extend())?;
    provider.get_artist(&id).await.map_err(|e| e.extend())
}
```

- [ ] **Step 4: Remove dead helper functions from `schema.rs`**

Delete:
- `search_sources()`
- `SOURCE_QUERY_TIMEOUT` const
- `search_musicbrainz_artists`, `search_audius_artists`
- `search_musicbrainz_tracks`, `search_audius_tracks`
- `random_musicbrainz_artists`, `random_musicbrainz_tracks`
- `random_audius_artists`, `random_audius_tracks`
- `require_audius_client()`

- [ ] **Step 5: Update `src/graphql/mod.rs` re-exports**

Remove `require_podcast_service` and `require_audiobook_service` from re-exports. Add any new helpers needed (or inline them in the resolvers that need them).

- [ ] **Step 6: Refactor podcast resolvers in `src/graphql/podcast/queries.rs`**

Replace `require_podcast_service(app_ctx)?` with:
```rust
fn require_podcast_provider(app_ctx: &AppContext) -> GqlResult<&Arc<dyn PodcastProvider>> {
    app_ctx.podcast_providers.first()
        .ok_or_else(|| AppError::FeatureDisabled(
            "Podcasts not configured. Set podcast_index in config.yaml.".into()
        ).extend())
}
```

Update all methods to call `provider.search_podcasts(...)`, `provider.trending(...)`, etc.

For single-entity lookups (`podcast`, `episode`), keep the existing `PodcastSource::parse_id()` logic, find the matching provider, and call it.

- [ ] **Step 7: Refactor audiobook resolvers in `src/graphql/audiobook/queries.rs`**

Same pattern. Replace `require_audiobook_service(app_ctx)?` with:
```rust
fn require_audiobook_provider(app_ctx: &AppContext) -> GqlResult<&Arc<dyn AudiobookProvider>> {
    app_ctx.audiobook_providers.first()
        .ok_or_else(|| AppError::FeatureDisabled(
            "LibriVox is not configured. Set librivox in config.yaml.".into()
        ).extend())
}
```

Remove `random_librivox_audiobooks` function (now in LibriVoxProvider).

Update `random_audiobooks` resolver to call `provider.random_audiobooks(limit)`.

- [ ] **Step 8: Refactor unified queries in `src/graphql/unified/queries.rs`**

Replace domain helpers with provider-based fan-out:

```rust
async fn search_music(
    providers: &[Arc<dyn MusicProvider>],
    query: &str, limit: i32,
) -> std::result::Result<Option<MusicSearchResults>, AppError> {
    if providers.is_empty() { return Ok(None); }
    let (artists, tracks) = tokio::join!(
        fan_out_search(providers, SOURCE_TIMEOUT, |p| {
            let q = query.to_string();
            async move { p.search_artists(&q, limit, 0).await }
        }),
        fan_out_search(providers, SOURCE_TIMEOUT, |p| {
            let q = query.to_string();
            async move { p.search_tracks(&q, limit, 0).await }
        }),
    );
    Ok(Some(MusicSearchResults { artists, tracks }))
}
```

Similarly for `search_podcasts`, `search_audiobooks`, `random_music`, `random_podcasts`, `random_audiobooks`.

Remove `gql_to_app_error` (no longer needed).

Update `search` and `random` resolvers to pass `&app_ctx.music_providers`, `&app_ctx.podcast_providers`, `&app_ctx.audiobook_providers`.

- [ ] **Step 9: Update `test_schema_builds` test in `schema.rs`**

The existing test constructs `AppContext` with old fields. Update it:

```rust
#[test]
fn test_schema_builds() {
    let app_context = AppContext {
        music_providers: vec![],
        podcast_providers: vec![],
        audiobook_providers: vec![],
    };
    let _schema = build_schema(app_context);
}
```

- [ ] **Step 10: Run full check**

Run: `task check`
Expected: Lint clean, all tests pass.

- [ ] **Step 11: Commit**

```
[Abstraction]: Rewire AppContext and all resolvers to use provider traits
```

---

## Task 5: Jamendo Client

**Files:**
- Create: `src/jamendo/mod.rs`
- Create: `src/jamendo/client.rs`
- Create: `src/jamendo/types.rs`
- Modify: `src/lib.rs`
- Modify: `src/config.rs`

Build the Jamendo HTTP client. API docs: https://developer.jamendo.com/v3.0

- [ ] **Step 1: Add Jamendo config**

In `src/config.rs`, add:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct JamendoConfig {
    #[serde(default)]
    pub enabled: bool,
    pub client_id: Option<String>,
    #[serde(default = "default_jamendo_base_url")]
    pub base_url: String,
}

fn default_jamendo_base_url() -> String {
    "https://api.jamendo.com/v3.0".to_string()
}
```

Add `pub jamendo: Option<JamendoConfig>` to `AppConfig`.

- [ ] **Step 2: Define Jamendo API response types**

Create `src/jamendo/types.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct JamendoResponse<T> {
    pub headers: JamendoHeaders,
    pub results: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct JamendoHeaders {
    pub status: String,
    pub code: i32,
    pub results_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JamendoTrack {
    pub id: String,
    pub name: String,
    pub duration: i32,
    pub artist_id: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub album_id: Option<String>,
    pub audio: String,
    pub audiodownload: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JamendoArtist {
    pub id: String,
    pub name: String,
    pub website: Option<String>,
    pub image: Option<String>,
    pub joindate: Option<String>,
}
```

- [ ] **Step 3: Implement JamendoClient**

Create `src/jamendo/client.rs`:

```rust
use crate::error::Result;
use crate::http::ApiClient;
use super::types::*;

pub struct JamendoClient {
    client: ApiClient,
    client_id: String,
}

impl JamendoClient {
    pub fn new(client_id: String, base_url: &str) -> Result<Self> { ... }

    pub async fn search_tracks(&self, query: &str, limit: i32, offset: i32)
        -> Result<Vec<JamendoTrack>> { ... }

    pub async fn search_artists(&self, query: &str, limit: i32, offset: i32)
        -> Result<Vec<JamendoArtist>> { ... }

    pub async fn get_track(&self, id: &str) -> Result<Option<JamendoTrack>> { ... }

    pub async fn get_artist(&self, id: &str) -> Result<Option<JamendoArtist>> { ... }

    pub async fn popular_tracks(&self, limit: i32) -> Result<Vec<JamendoTrack>> { ... }
}
```

All methods append `client_id=X&format=json` to every request. Search uses `search=Q`, artist search uses `namesearch=Q`. Popular tracks use `order=popularity_total`.

- [ ] **Step 4: Add Jamendo GraphQL types**

In `src/graphql/types.rs`, add a `JamendoArtist` struct and `JamendoTrack` struct with `#[Object]` impls that satisfy the interface fields:

```rust
// New struct:
pub struct JamendoArtist {
    pub id: String,
    pub name: String,
    pub source: DataSource,
    pub source_id: String,
    pub image_url: Option<String>,
    pub website: Option<String>,
}

#[Object]
impl JamendoArtist {
    async fn id(&self) -> &str { &self.id }
    async fn name(&self) -> &str { &self.name }
    async fn source(&self) -> DataSource { self.source }
    async fn source_id(&self) -> &str { &self.source_id }
    async fn image_url(&self) -> Option<&str> { self.image_url.as_deref() }
    async fn website(&self) -> Option<&str> { self.website.as_deref() }
}

// New struct:
pub struct JamendoTrack {
    pub id: String,
    pub title: String,
    pub source: DataSource,
    pub source_id: String,
    pub duration_ms: Option<i64>,
    pub artist_name: Option<String>,
    pub audio_url: String,
    pub image_url: Option<String>,
    pub album_name: Option<String>,
}

#[Object]
impl JamendoTrack {
    async fn id(&self) -> &str { &self.id }
    async fn title(&self) -> &str { &self.title }
    async fn source(&self) -> DataSource { self.source }
    async fn source_id(&self) -> &str { &self.source_id }
    async fn duration_ms(&self) -> Option<i64> { self.duration_ms }
    async fn artist_name(&self) -> Option<&str> { self.artist_name.as_deref() }
    async fn audio_url(&self) -> &str { &self.audio_url }
    async fn image_url(&self) -> Option<&str> { self.image_url.as_deref() }
    async fn album_name(&self) -> Option<&str> { self.album_name.as_deref() }
}
```

Add variants to the interface enums:
```rust
pub enum Artist {
    MusicBrainz(MusicBrainzArtist),
    Audius(AudiusArtist),
    Jamendo(JamendoArtist),   // NEW
}

pub enum Track {
    MusicBrainz(MusicBrainzTrack),
    Audius(AudiusTrack),
    Jamendo(JamendoTrack),    // NEW
}
```

Ensure the new structs implement all required interface fields (check the `#[graphql(Interface)]` definition).

Add `From<jamendo::types::JamendoTrack>` and `From<jamendo::types::JamendoArtist>` conversions that create the GraphQL types with source-prefixed IDs.

- [ ] **Step 5: Wire module**

Create `src/jamendo/mod.rs`:
```rust
mod client;
mod types;

pub use client::JamendoClient;
pub use types::{JamendoArtist, JamendoTrack};
```

Add `pub mod jamendo;` to `src/lib.rs`.

- [ ] **Step 6: Write unit tests**

Test JSON deserialization of Jamendo API responses. Test type conversions (JamendoTrack → GraphQL Track, JamendoArtist → GraphQL Artist).

- [ ] **Step 7: Run tests**

Run: `task check`

- [ ] **Step 8: Commit**

```
[Jamendo]: Add Jamendo API client, types, and GraphQL type variants
```

---

## Task 6: Implement JamendoProvider and Wire Into Server

**Files:**
- Create: `src/sources/jamendo.rs`
- Modify: `src/sources/mod.rs`
- Modify: `src/server.rs`
- Modify: `config.yaml`

- [ ] **Step 1: Implement JamendoProvider**

Create `src/sources/jamendo.rs`:

```rust
use crate::domain::{DataSource, MusicProvider};
use crate::error::Result;
use crate::graphql::helpers::random_sample;
use crate::graphql::types::{Artist, Track};
use crate::jamendo::JamendoClient;

pub struct JamendoProvider {
    client: JamendoClient,
}

impl JamendoProvider {
    pub fn new(client: JamendoClient) -> Self {
        Self { client }
    }
}

impl MusicProvider for JamendoProvider {
    fn source_id(&self) -> DataSource { DataSource::Jamendo }

    async fn search_artists(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Artist>> {
        let results = self.client.search_artists(query, limit, offset).await?;
        Ok(results.into_iter().map(|a| Artist::Jamendo(a.into())).collect())
    }

    async fn search_tracks(&self, query: &str, limit: i32, offset: i32) -> Result<Vec<Track>> {
        let results = self.client.search_tracks(query, limit, offset).await?;
        Ok(results.into_iter().map(|t| Track::Jamendo(t.into())).collect())
    }

    async fn get_artist(&self, id: &str) -> Result<Option<Artist>> {
        let result = self.client.get_artist(id).await?;
        Ok(result.map(|a| Artist::Jamendo(a.into())))
    }

    async fn get_track(&self, id: &str) -> Result<Option<Track>> {
        let result = self.client.get_track(id).await?;
        Ok(result.map(|t| Track::Jamendo(t.into())))
    }

    async fn random_artists(&self, limit: i32) -> Result<Vec<Artist>> {
        let tracks = self.client.popular_tracks(limit * 3).await?;
        let mut seen = std::collections::HashSet::new();
        let artists: Vec<Artist> = tracks.into_iter()
            .filter(|t| seen.insert(t.artist_id.clone()))
            .map(|t| {
                // Build a minimal JamendoArtist from track data
                Artist::Jamendo(crate::graphql::types::JamendoArtist {
                    id: DataSource::Jamendo.format_id(&t.artist_id),
                    name: t.artist_name.clone(),
                    source: DataSource::Jamendo,
                    source_id: t.artist_id,
                    image_url: t.image.clone(),
                    website: None,
                })
            })
            .collect();
        Ok(random_sample(artists, limit as usize))
    }

    async fn random_tracks(&self, limit: i32) -> Result<Vec<Track>> {
        let tracks = self.client.popular_tracks(limit * 2).await?;
        let tracks: Vec<Track> = tracks.into_iter()
            .map(|t| Track::Jamendo(t.into()))
            .collect();
        Ok(random_sample(tracks, limit as usize))
    }
}
```

- [ ] **Step 2: Export in sources/mod.rs**

Add:
```rust
mod jamendo;
pub use jamendo::JamendoProvider;
```

- [ ] **Step 3: Wire into server.rs**

```rust
use crate::sources::JamendoProvider;
use crate::jamendo::JamendoClient;

// After building music_providers vec:
if let Some(ref jamendo_config) = config.jamendo {
    if jamendo_config.enabled {
        let client_id = jamendo_config.client_id.clone()
            .or_else(|| std::env::var("JAMENDO_CLIENT_ID").ok())
            .expect("JAMENDO_CLIENT_ID required when jamendo is enabled");
        let client = JamendoClient::new(client_id, &jamendo_config.base_url)?;
        music_providers.push(Arc::new(JamendoProvider::new(client)));
        tracing::info!("Jamendo music source enabled");
    }
}
```

- [ ] **Step 4: Add to config.yaml**

```yaml
jamendo:
  enabled: true
  client_id: ${JAMENDO_CLIENT_ID}
```

- [ ] **Step 5: Run full check**

Run: `task check`

- [ ] **Step 6: Commit**

```
[Jamendo]: Implement JamendoProvider and wire into server
```

---

## Task 7: Smoke Tests, API Docs, and Final Verification

**Files:**
- Modify: `Taskfile.yml`
- Modify: `API.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add Jamendo smoke tests to Taskfile.yml**

```bash
run_query "search_artists_jamendo" \
  '{"query":"{ searchArtists(source: JAMENDO, query: \"electronic\", limit: 3) { __typename ... on JamendoArtist { id name source } } }"}'

run_query "search_tracks_jamendo" \
  '{"query":"{ searchTracks(source: JAMENDO, query: \"ambient\", limit: 3) { __typename ... on JamendoTrack { id title source audioUrl } } }"}'
```

- [ ] **Step 2: Update API.md**

Add `JAMENDO` to the `DataSource` enum docs. Add Jamendo examples to `searchArtists` and `searchTracks`. Document `JamendoArtist` and `JamendoTrack` fields (especially `audioUrl` for streaming).

- [ ] **Step 3: Update CHANGELOG.md**

Under `### Added`:
- Source provider abstraction with `MusicProvider`, `PodcastProvider`, `AudiobookProvider` traits and generic `fan_out_search`
- Jamendo integration as a new music source with search, browse, and streaming URLs

- [ ] **Step 4: Run `task check`**

Lint + unit tests must pass.

- [ ] **Step 5: Start server and run `task test:auth`**

All existing tests + new Jamendo tests must pass.

- [ ] **Step 6: Manual verification**

Test unified search returns results from all 3 music sources:
```graphql
{ search(query: "electronic") {
    music { artists { __typename } tracks { __typename } }
}}
```

Test source filtering:
```graphql
{ searchTracks(query: "ambient", source: JAMENDO, limit: 5) {
    __typename ... on JamendoTrack { id title audioUrl }
}}
```

- [ ] **Step 7: Commit**

```
[Docs]: Add Jamendo to smoke tests, API docs, and CHANGELOG
```
