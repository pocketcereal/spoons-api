# Code Review Issues

## Issues with Clear Solutions - **DONE**

### 1. Unused `redis` dependency - **DONE**
Removed redis dependency from Cargo.toml - not used anywhere in codebase.

### 2. Dead `cache` module - **DONE**
Deleted src/cache/ module and removed from lib.rs.

### 3. `is_request()` too broad in retry logic - **DONE**
Fixed src/http/client.rs - removed is_request() from is_retryable_error().

### 4. Response error text handling - **DONE**
Fixed src/http/client.rs - improved error message when response body read fails.

### 5. GraphQL GET support missing - **DONE**
Added GET handler to GraphQL route for introspection queries.

### 6. Error mapping helper - **DONE**
Created `db_error()` helper in src/db/helpers.rs to reduce repetition.

---

## Issues Requiring Design Decisions

### 7. Repository Pattern Duplication (CRITICAL)

**Description:** All four repository files (artist.rs, recording.rs, release.rs, release_group.rs) follow identical patterns with 90%+ code duplication including:
- Same `MAX_BATCH_SIZE = 100` constant
- Identical `get_cached()`, `get_by_id()`, `get_by_ids()` signatures
- Same `upsert()` and `upsert_many()` transaction patterns

**Options:**
- **Option A: Trait-based generic repository** - Create `Repository<T>` trait with associated types for Row, NewRow, and Entity. Implement common operations once.
- **Option B: Macro-based generation** - Use declarative macros to generate the repetitive code.
- **Option C: Keep as-is with documentation** - Accept duplication for explicitness and ease of debugging.

#### Likely Best Solution
**Option A (Trait-based)** - Provides type safety, IDE support, and 40-50% code reduction while maintaining clarity. Macros can become hard to debug.

---

### 8. Search Cache Duplication (CRITICAL)

**Description:** search_cache.rs has 8 nearly-identical methods (get/cache for artist, release, recording, release_group). Each pair follows the exact same pattern differing only in table name and entity type.

**Options:**
- **Option A: Generic `Cacheable` trait** - Define trait with associated types for cache row, entity, and ID extraction.
- **Option B: Macro generation** - Generate the repetitive methods with a macro.

#### Likely Best Solution
**Option A (Cacheable trait)** - Aligns with repository refactor, maintains type safety.

---

### 9. Database Pool Configuration

**Description:** Pool builder only sets `max_size()`. Missing min_idle, wait_timeout, and health check configuration which are important for production.

**Options:**
- **Option A: Add full configuration** - Extend DbConfig with all pool settings.
- **Option B: Sensible defaults** - Add hardcoded sensible defaults without config exposure.

#### Likely Best Solution
**Option B** for now - add sensible defaults. Can expose to config later if needed.

---

### 10. MusicBrainz Query Parameter Inconsistency

**Description:** Some methods use `?fmt=json` directly in the path string, while the search methods use a params struct. Should be consistent.

**Options:**
- **Option A: All use params struct** - Create `FormatParam { fmt: "json" }` for single-entity fetches.
- **Option B: All use path string** - Simpler but less flexible.

#### Likely Best Solution
**Option A** - Consistency with search methods, easier to extend.

---

## Status Summary

| Issue | Status |
|-------|--------|
| 1. Unused redis dependency | **DONE** |
| 2. Dead cache module | **DONE** |
| 3. is_request() retry fix | **DONE** |
| 4. Response error handling | **DONE** |
| 5. GraphQL GET support | **DONE** |
| 6. Error mapping helper | **DONE** |
| 7. Repository duplication | **IN PROGRESS** - Phase 1 complete (shared constants) |
| 8. Search cache duplication | Needs larger refactor (blocked by #7) |
| 9. Database pool config | Minor - defer |
| 10. MusicBrainz params | Minor - defer |

## Refactor Progress

See `docs/REPOSITORY_REFACTOR.md` for detailed plan.

- [x] Phase 1: Extract shared constants and validation
- [ ] Phase 2: Extract cache query pattern
- [ ] Phase 3: Extract batch query pattern
- [ ] Phase 4: Extract batch upsert pattern
- [ ] Phase 5: Cleanup and documentation
