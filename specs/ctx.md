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
  - value fields: `rows`, `columns`, `total`, `total_all`
  - behavior: read-through (hit returns cached value, miss computes + inserts)
  - startup prewarm: defaults for both tables (`page=1`, `limit=50`, no sort/filter/columns)
  - startup prewarm policy: fail-fast if either prewarm call fails

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
  - prewarm default table entries before listener bind
  - in `src/main.rs`
- Request path wiring:
  - `get_stats()` now returns `overview_stats`
  - table server functions use cached wrappers
  - REST table handlers use cached wrappers
  - in `src/server/functions.rs`, `src/server/common.rs`, `src/server/handlers.rs`
- Issue #15 metadata delivery refactor (resolved):
  - server injects metadata JSON in global shell (`<script type="application/json">`)
  - client initializes shared metadata store once at app startup
  - table server-function responses are data-only (no metadata field)
  - table cache values are data-only (no metadata field)
  - table components read metadata from global hydrated store for selector/tooltips
  - metadata stays available on client navigation from `/` to table pages
  - in `src/app.rs`, `src/main.rs`, `src/metadata.rs`, `src/server/cache.rs`, `src/server/common.rs`, `src/server/functions.rs`, `src/components/*table*.rs`, `src/table/table.rs`, `src/components/column_selector.rs`
- SSR hydration stability fix:
  - moved metadata JSON script injection into `<head>` to avoid body hydration marker mismatch
  - avoids `failed_to_cast_marker_node` / unrecoverable hydration panic caused by extra body node
- Added cache tests:
  - key normalization tests in `src/server/cache.rs`
  - miss/hit behavior in `src/server/common.rs`
  - prewarm key population checks in `src/server/common.rs`
  - restart semantics (fresh cache starts empty) in `src/server/common.rs`
- Baseline e2e test integration (local):
  - Playwright smoke suite added under `end2end/tests/smoke.spec.ts`
  - 3 tests:
    - SSR + hydration `/stellarhosts`
    - SSR + hydration `/exoplanets`
    - `/` -> client navigation to `/stellarhosts` keeps metadata available
  - readiness guard added to avoid startup race (`ERR_CONNECTION_REFUSED`)
  - Playwright config narrowed to deterministic local baseline (`chromium`, `workers=1`, `fullyParallel=false`)
  - README section added for e2e setup/run/report workflow
- Build fix:
  - `cargo leptos build` wasm fix for `uuid` (`js` feature) in `Cargo.toml`

## Pending
- Formalize post-fix verification coverage:
  - scripted checks for explicit-query SSR routes and plain SSR routes in one pass
  - scripted checks for REST table endpoints in same verification run
- Expand e2e coverage incrementally:
  - payload regression checks (table responses remain metadata-free)
  - optional CI integration after local flow is stable for team usage
- Optional cleanup:
  - move cache size `400` to env/config
  - evaluate `moka::future::Cache::get_with` to deduplicate concurrent misses

## Next Session Summary
Start from these concrete tasks:

1. Re-validate cache behavior after Task 0 changes:
- `cargo check`
- targeted server/cache tests
- startup log shows prewarm completion before listener bind

2. Add scripted runtime verification:
- plain SSR routes: `/stellarhosts`, `/exoplanets`
- explicit-query SSR routes for both pages
- REST endpoints: `/rest/stellarhosts`, `/rest/exoplanets`
- confirm no chunked-encoding SSR failures in these checks

3. Extend e2e suite (post-baseline):
- add payload regression assertions (metadata not present in table responses)
- consider CI wiring after local baseline remains stable
