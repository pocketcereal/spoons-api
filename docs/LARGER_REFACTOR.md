# Larger Refactors Needed

## 1. Repository Pattern Duplication (CRITICAL)

### Problem
All four repository files (artist.rs, recording.rs, release.rs, release_group.rs) have 90%+ code duplication with identical:
- `MAX_BATCH_SIZE = 100` constant
- `get_cached()` method pattern
- `get_by_id()` method pattern
- `get_by_ids()` with batch validation
- `upsert()` transaction pattern
- `upsert_many()` batch operation

### Why This Needs a Larger Refactor
- Changes to the pattern require updating 4 files
- Bug fixes must be replicated across all repositories
- Testing the same logic 4 times
- ~600 lines of nearly-identical code

### Recommended Solution: Trait-based Repository

Create a generic trait with associated types:

```rust
pub trait Repository {
    type Entity;
    type Row: Selectable<Pg> + Queryable</* ... */>;
    type NewRow: Insertable</* ... */>;

    fn table() -> /* table type */;

    async fn get_cached(pool: &DbPool, id: &str, ttl: i64) -> Result<Option<Self::Entity>>;
    async fn get_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Self::Entity>>;
    async fn upsert(pool: &DbPool, entity: &Self::Entity) -> Result<()>;
    async fn upsert_many(pool: &DbPool, entities: &[Self::Entity]) -> Result<()>;
}
```

### Complexity
- Medium-high: Diesel's type system with async requires careful handling
- Estimated: 200-300 lines of generic code to replace 600+ lines of duplication

---

## 2. Search Cache Duplication (CRITICAL)

### Problem
search_cache.rs has 8 nearly-identical methods:
- `get_artist_search` / `cache_artist_search`
- `get_release_search` / `cache_release_search`
- `get_recording_search` / `cache_recording_search`
- `get_release_group_search` / `cache_release_group_search`

Each pair differs only in:
- Table name
- Entity type
- Repository used

### Why This Needs a Larger Refactor
- Same pattern repeated 4 times
- Any fix needs to be applied 4 times
- ~300 lines of nearly-identical code

### Recommended Solution: Generic Cacheable Trait

```rust
pub trait Cacheable {
    type Row;
    type CacheRow;
    type NewCacheRow;

    fn cache_table() -> /* table type */;
    fn to_ids(entities: &[Self]) -> Vec<Uuid>;
    async fn get_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Self>>;
    async fn upsert_many(pool: &DbPool, entities: &[Self]) -> Result<()>;
}

impl SearchCacheRepository {
    async fn get_cached<T: Cacheable>(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        cache_ttl_seconds: i64,
    ) -> Result<Option<Vec<T>>> {
        // Generic implementation
    }

    async fn cache<T: Cacheable>(
        pool: &DbPool,
        query: &str,
        limit: i32,
        offset: i32,
        entities: &[T],
    ) -> Result<()> {
        // Generic implementation
    }
}
```

### Complexity
- Medium: Depends on Repository refactor (issue #1)
- Should be done after Repository refactor
- Estimated: 100-150 lines of generic code to replace 300+ lines

---

## Recommended Order of Refactors

1. **Repository Trait Refactor** (this is the foundation)
2. **Search Cache Generic Trait** (builds on repository refactor)

Both refactors share a common dependency structure and should be planned together.
