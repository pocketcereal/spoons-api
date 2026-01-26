# Repository Pattern Refactor Plan

## Current State Analysis

### Files to Refactor
- `src/db/repositories/artist.rs` (229 lines)
- `src/db/repositories/recording.rs` (155 lines)
- `src/db/repositories/release.rs` (247 lines)
- `src/db/repositories/release_group.rs` (157 lines)

### Duplicated Code Patterns

1. **MAX_BATCH_SIZE constant** - identical in all 4 files
2. **get_cached()** - same signature, same TTL check logic
3. **get_by_id()** - same signature, same single-row fetch
4. **get_by_ids()** - same batch validation, same IN query pattern
5. **upsert()** - same transaction pattern with area handling (artist) or simpler
6. **upsert_many()** - same batch insert with on_conflict

### Dependencies (Code Path Analysis)

```
MusicRepository (facade in mod.rs)
    ├── ArtistRepository::get_cached()
    │   ├── parse_uuid() ← helpers.rs
    │   ├── get_conn() ← helpers.rs
    │   └── diesel query on artists table
    │
    ├── ArtistRepository::upsert()
    │   ├── get_conn()
    │   ├── Transaction with Box::pin()
    │   └── AreaRepository::upsert_if_present() (artist-specific)
    │
    └── ArtistRepository::upsert_many()
        ├── filter_map with UUID validation
        └── batch insert_into

SearchCacheRepository
    ├── ArtistRepository::get_by_ids()
    └── ArtistRepository::upsert_many()
```

### Key Differences Between Repositories

| Feature | Artist | Recording | Release | ReleaseGroup |
|---------|--------|-----------|---------|--------------|
| Has related entity (area) | Yes | No | No | No |
| Row type | ArtistRow | RecordingRow | ReleaseRow | ReleaseGroupRow |
| NewRow type | NewArtistRow | NewRecordingRow | NewReleaseRow | NewReleaseGroupRow |
| Entity type | Artist | Recording | Release | ReleaseGroup |
| Table | artists | recordings | releases | release_groups |
| Complex upsert | Yes (area FK) | No | No | No |

---

## Refactor Strategy

### Option A: Trait with Associated Types (CHOSEN)

Create a `CacheRepository` trait that defines the common interface:

```rust
pub trait CacheRepository {
    type Entity: Clone;
    type Row: Queryable</* diesel types */> + Selectable<Pg>;
    type NewRow: Insertable</* table */>;

    // Constants
    const MAX_BATCH_SIZE: usize = 100;

    // Required implementations
    fn table() -> /* schema table */;
    fn row_to_entity(row: Self::Row) -> Self::Entity;
    fn entity_to_new_row(entity: &Self::Entity) -> Result<Self::NewRow>;

    // Default implementations (provided)
    async fn get_cached(pool: &DbPool, id: &str, ttl: i64) -> Result<Option<Self::Entity>>;
    async fn get_by_ids(pool: &DbPool, ids: &[Uuid]) -> Result<Vec<Self::Entity>>;
    async fn upsert_many(pool: &DbPool, entities: &[Self::Entity]) -> Result<()>;
}
```

### Challenges with Diesel + Async

1. **Table type references** - Diesel uses type-level schema, need careful type bounds
2. **Query builder generics** - Complex type signatures for queries
3. **Async closures in transactions** - Lifetime issues with Box::pin

### Simpler Alternative: Helper Functions + Macros

Instead of full trait implementation, extract common code into helpers:

```rust
// helpers.rs
pub async fn cached_query<Row, F>(
    pool: &DbPool,
    id: &str,
    ttl: i64,
    query_fn: F,
) -> Result<Option<Row>>
where
    F: FnOnce(&mut AsyncPgConnection, Uuid, DateTime<Utc>) -> /* query future */
{
    let uuid = parse_uuid(id)?;
    let min_cached_at = Utc::now() - Duration::seconds(ttl);
    let mut conn = get_conn(pool).await?;
    query_fn(&mut conn, uuid, min_cached_at).await
}

// Plus macro for generating repetitive methods
macro_rules! impl_cache_repository {
    ($repo:ident, $table:ident, $row:ty, $entity:ty) => {
        // Generated implementations
    }
}
```

---

## Step-by-Step Refactor Plan

### Phase 1: Extract Shared Constants and Validation
1. Move `MAX_BATCH_SIZE` to helpers.rs
2. Create `validate_batch_size()` helper
3. Apply to all 4 repositories
4. Run `task check`
5. Commit: `[refactor]: repository-shared-constants`

### Phase 2: Extract Cache Query Pattern
1. Create `cached_by_id_query()` helper function
2. Refactor `get_cached()` in artist.rs to use it
3. Test, then apply to other 3 repositories
4. Run `task check`
5. Commit: `[refactor]: repository-cached-query-helper`

### Phase 3: Extract Batch Query Pattern
1. Create `get_by_ids_query()` helper
2. Refactor `get_by_ids()` in all repositories
3. Run `task check`
4. Commit: `[refactor]: repository-batch-query-helper`

### Phase 4: Extract Batch Upsert Pattern
1. Create generic `batch_upsert()` helper (most complex)
2. Handle the artist-specific area logic separately
3. Refactor `upsert_many()` in all repositories
4. Run `task check`
5. Commit: `[refactor]: repository-batch-upsert-helper`

### Phase 5: Cleanup and Documentation
1. Remove any remaining duplication
2. Update module documentation
3. Ensure all tests pass
4. Commit: `[refactor]: repository-cleanup`

---

## Code Paths Affected

Files that call repository methods (must remain working):
- `src/db/repositories/mod.rs` - MusicRepository facade
- `src/db/repositories/search_cache.rs` - SearchCacheRepository
- `src/graphql/schema.rs` - GraphQL resolvers
- `tests/integration/` - Integration tests

---

## Success Criteria

- [ ] All 25 unit tests pass
- [ ] All integration tests pass (if DB available)
- [ ] `task check` passes (lint + test)
- [ ] Code reduction: target 40%+ less lines in repository files
- [ ] No behavioral changes to API
