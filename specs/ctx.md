# Current Context

## Goal
Implement backend-only in-process caching so overview and table requests avoid repeated heavy computations.

## Key Decisions
- Cache library: `moka` (`future` feature), server side only.
- Frontend does not use `moka`; keep lightweight Leptos resource/signal state.
- Cache invalidation model: process-local cache, rebuilt on service restart.
- No dedicated metadata endpoint in this phase.

## Cache Design (Current)
- Overview cache:
  - precompute `DataStats` at startup
  - store in `ApiState.overview_stats`
  - `get_stats()` returns cached value directly
- Table cache:
  - bounded in-memory cache (`max_entries = 400`)
  - key fields: `table`, `page`, `limit`, `sort_by`, `order`, `columns`, `filter`
  - key normalization: trim/lowercase for sort/filter, normalize empty values, keep column order
  - value fields: `rows`, `columns`, `total`, `total_all`, `metadata`
  - behavior: read-through (hit returns cached value, miss computes + inserts)

## Implemented
- Added cache module and types:
  - `src/server/cache.rs`
  - `src/server/mod.rs` exports `cache`
- Added `ApiState` cache fields:
  - `overview_stats`
  - `table_cache`
  - in `src/server/handlers.rs`
- Startup wiring:
  - precompute overview stats
  - build table cache
  - in `src/main.rs`
- Request path wiring:
  - `get_stats()` now returns `overview_stats`
  - table server functions use cached wrappers
  - REST table handlers use cached wrappers
  - in `src/server/functions.rs`, `src/server/common.rs`, `src/server/handlers.rs`
- Added normalization tests in `src/server/cache.rs`.

## Pending
- Required startup prewarm before serving requests:
  - overview is precomputed already
  - still need prewarm for initial table pages (`page=1`, `limit=50`, default columns, no sort/filter)
- Additional tests:
  - explicit hit/miss cache behavior
  - restart rebuild behavior
  - prewarm population checks

## Next Session Summary
Start from these concrete tasks:

1. Implement startup prewarm in `src/main.rs`:
- call cached table loaders for both tables using initial query parameters
- run before server starts listening

2. Add tests for cache behavior:
- first request miss + second request hit (same normalized key)
- prewarm inserts expected keys
- restart semantics (new empty cache on new `ApiState`)

3. Verify acceptance:
- `cargo check`
- targeted server tests
- optional manual endpoint checks for both `/rest/*` and server functions

4. Optional cleanup:
- move cache size `400` to env/config
- decide whether to use `moka::future::Cache::get_with` to avoid duplicate concurrent miss work
