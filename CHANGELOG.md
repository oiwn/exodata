# Changelog

## 2026-02-13

- Resolved metadata delivery overhead (Issue #15) by moving table metadata to one-time global hydration:
  - Added shared metadata store/context in `src/metadata.rs`.
  - Injected metadata JSON in SSR shell and initialized store app-wide (`src/app.rs`, `src/main.rs`).
  - Kept metadata available across client navigation (including `/` -> table pages).
- Converted table request path to data-only payloads:
  - Removed metadata from `TableData` responses in `src/server/functions.rs`.
  - Removed metadata from cached table values in `src/server/cache.rs`.
  - Updated shared table loaders and REST handlers for data-only tuples (`src/server/common.rs`, `src/server/handlers.rs`).
  - Updated table/selector components to read metadata from global store instead of per-response payloads.
- Fixed hydration mismatch caused by metadata script placement:
  - moved metadata script injection into `<head>` to avoid body hydration marker conflicts.
- Added local Playwright e2e baseline:
  - Added smoke suite (`end2end/tests/smoke.spec.ts`) with 3 flows:
    - SSR + hydration `/stellarhosts`
    - SSR + hydration `/exoplanets`
    - `/` -> client navigation to `/stellarhosts` with metadata-backed column selector
  - Added server readiness guard to avoid startup race (`ERR_CONNECTION_REFUSED`).
  - Simplified local Playwright config to deterministic baseline (`chromium`, single worker, sequential).
- Documented e2e setup/run/report workflow in `README.md`.

## 2026-02-12

- Added backend cache wiring for overview and table responses:
  - Added server cache module (`src/server/cache.rs`) and exported it from `src/server/mod.rs`.
  - Added `overview_stats` and `table_cache` to `ApiState`.
  - Updated startup to precompute overview stats and initialize table cache.
  - Switched server functions and REST handlers to cached table loaders.
- Added cache test coverage in `src/server/common.rs`:
  - miss->hit behavior for normalized keys
  - prewarm key population checks
  - restart semantics with fresh cache
- Fixed `cargo leptos build` failure on wasm by enabling `uuid` JS RNG support in `Cargo.toml`.
- Fixed table-page SSR instability by preventing no-op metadata signal writes in:
  - `src/components/exoplanets_table.rs`
  - `src/components/stellarhosts_table.rs`
- Added `PartialEq`/`Eq` derive for `ColumnMetadata` in `src/server/functions.rs` to support metadata equality checks.
