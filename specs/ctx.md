# Current Context

## Current Task

Design and implement backend caching so the app does not recompute expensive overview and table work on every request.

## Why This Is Needed

Current behavior:
- DataFrames and metadata are loaded once at startup.
- Overview stats are recomputed on each `get_stats()` call.
- Table queries recompute `select/filter/sort/paginate` on each request.
- Full table metadata is returned on every table response.

This causes avoidable CPU work and repeated serialization.

## Scope

In scope:
- Precomputed overview cache at startup.
- Bounded server-side cache for table query results.
- Table cache prewarm for hot/default queries.
- Cache invalidation strategy tied to dataset version.
- Basic cache observability (hit/miss/latency).

Out of scope (for this phase):
- Redis or distributed cache.
- Cross-instance shared cache.
- Live data hot-reload without restart.
- Full query-engine redesign.

## Cache Model

### 1) Overview Cache (Materialized at Startup)

- Compute `DataStats` once during server startup.
- Store as immutable shared state (`Arc<DataStats>`).
- `get_stats()` returns cached value directly (no per-request aggregation).
- Recompute only when dataset version changes (currently: server restart).

### 2) Table Query Cache (Bounded In-Memory)

Use a bounded cache keyed by normalized query parameters.

Key fields:
- `dataset_version` (required)
- `table` (`stellarhosts` or `exoplanets`)
- `page`
- `limit`
- `sort_by` (or null marker)
- `order` (`asc` or `desc`)
- `columns` (validated column list, order-preserving)
- `filter` (trimmed and lowercased)

Value fields:
- `rows` (serialized JSON rows)
- `columns` (response columns)
- `total`
- `total_all`
- `metadata` (phase 1: keep for compatibility)

Cache behavior:
- Read-through on request.
- Miss: compute response, store, return.
- Hit: return cached response directly.
- Eviction: LRU + max entries (or max weight/bytes).
- Optional idle TTL to prevent stale memory growth.

Initial limits:
- `max_entries`: 400 (tunable by env).
- `limit` request cap remains unchanged.

### 3) Prewarm Strategy

At startup, precompute and cache:
- Overview stats.
- Default page for both tables:
  - `page=1`, `limit=50`
  - default columns
  - no sort
  - empty filter

Optional prewarm (configurable):
- pages 2-3 for both tables.

## Invalidation

Primary rule:
- Any dataset change must invalidate all caches.

Mechanism:
- Cache keys include `dataset_version`.
- `dataset_version` is derived from data artifacts (for example parquet file mtimes or hash at startup).
- On startup with new data, new version naturally bypasses old entries.
- Full flush is allowed on version change.

Current operational model:
- Data updates happen out-of-band, followed by service restart.

## API and Contract Notes

Phase 1:
- Keep current response contract unchanged.
- Keep metadata included in table responses to avoid frontend breakage.

Phase 2 (optimization):
- Add metadata-only endpoint (or one-time metadata fetch server function).
- Table responses omit metadata by default.
- Frontend stores metadata once per table.

## Concurrency and Runtime Safety

- DataFrames stay immutable and shared via `Arc`.
- Cache container must be thread-safe.
- Expensive miss computation should run in a blocking-safe path when needed, to avoid async runtime starvation under load.

## Observability

Add metrics/logging:
- `overview_cache_hits`, `overview_cache_misses` (misses should be near zero post-startup).
- `table_cache_hits`, `table_cache_misses`, `table_cache_evictions`.
- `table_compute_ms` (cache miss computation latency).
- `table_cache_entry_count`.

Add startup logs:
- dataset version
- prewarm start/end and duration
- prewarmed key count

## Acceptance Criteria

- `get_stats()` no longer runs aggregation functions per request.
- Repeated identical table queries are served from cache.
- Startup prewarm creates entries for overview and default table pages.
- Cache invalidates correctly when dataset version changes.
- No API response shape regression in phase 1.
- Add tests for key normalization, hit/miss behavior, and invalidation by dataset version.

## Implementation Plan

1. Add cache state to `ApiState`:
- `overview_stats: Arc<DataStats>`
- `table_cache: <thread-safe bounded cache>`
- `dataset_version: String`

2. Compute `overview_stats` and `dataset_version` at startup.

3. Add table cache wrapper in shared server logic:
- normalize input
- build key
- hit fast-path
- miss compute and insert

4. Add startup prewarm task for default queries.

5. Add metrics/logging hooks.

6. Add tests:
- normalization equivalence
- hit/miss correctness
- version-based invalidation
- prewarm population
