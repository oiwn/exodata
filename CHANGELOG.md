# Changelog

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
