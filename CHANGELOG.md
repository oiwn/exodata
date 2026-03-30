# Changelog

## 2026-03-29

- Added `tracing` instrumentation to server-side data path (Task 1):
  - Added `tracing-subscriber` with `env-filter` feature, gated behind `ssr` (`Cargo.toml`)
  - Initialized subscriber in `src/main.rs` (defaults to `info`, overridable via `RUST_LOG`)
  - Replaced `println!` with `tracing::info!` in `src/main.rs`
  - Added entry/exit `info!` and error-path `error!` to all 6 server functions (`src/server/functions.rs`)
  - Added cache hit/miss `debug!` to both `get_stellarhosts_data_cached` and `get_exoplanets_data_cached` (`src/server/common.rs`)
- Reduced WASM initial load by 55% (1.2 MB → 535 KB) via lazy route code splitting:
  - Switched `hydrate_body` → `hydrate_lazy` (`src/lib.rs`)
  - Added `#[lazy_route]` wrappers for all 6 routes (`src/components/*.rs`)
  - Updated route declarations to `Lazy::<X>::new()` (`src/app.rs`)
  - Build command now uses `--split` flag (`infrastructure/docker/Dockerfile`)
- Added code coverage via `cargo-llvm-cov` + Codecov (`.github/workflows/coverage.yml`):
  - Runs on push/PR to `main` and manual dispatch
  - Uses `codecov/codecov-action@v5` with `CODECOV_TOKEN` secret
  - Excludes frontend components, app shell, and metadata from coverage

## 2026-03-11

- Fixed hydration gap (Issue #26): users could interact with SSR content before WASM hydration completed
  - Added inline `<script>` to `shell()` head that sets `pre-hydration` class on `<html>` before body parses (`src/app.rs`)
  - CSS blocks interaction and shows dark overlay + spinner via `body::before` / `body::after` while class is present (`style/tailwind.css`)
  - WASM removes the class after `hydrate_body()` completes (`src/lib.rs`)
  - Added `web-sys` dependency scoped to `hydrate` feature only (`Cargo.toml`)
- **Fixed SSR streaming deadlock on 1-vCPU servers** (see `specs/ssr-streaming-issue.md`):
  - Root cause: Tokio defaulted to 1 worker thread on 1-vCPU droplet; Leptos SSR + `spawn_blocking` caused worker thread starvation
  - Fix: forced `worker_threads = 4` in `#[tokio::main]` (`src/main.rs`) so OS scheduler can interleave threads
  - Also changed `SsrMode::Async` → `SsrMode::OutOfOrder` for table routes (`src/app.rs`) to stream HTML shell immediately
- Updated Polars from 0.52 to 0.53 (`Cargo.toml`)
  - aligned `[dependencies]` and `[dev-dependencies]` to 0.53 with consistent feature flags
  - replaced removed `get_column_names_str()` with `get_column_names()` in `src/stellarhosts.rs`

## 2026-02-18 (Update 2)

- Adjusted SSR mode for table pages in `src/app.rs`:
  - set `/stellarhosts` route to `SsrMode::Async`
  - set `/exoplanets` route to `SsrMode::Async`
- Updated GitHub Actions flow:
  - `tests.yml` and `code-quality.yml` now run on `pull_request` (to `main`) and manual dispatch
  - `deploy.yml` now runs on `push` to `main` and manual dispatch
  - deploy remains gated by version bump detection in `Cargo.toml`

## 2026-02-18

- Added `tokio::task::spawn_blocking` for table cache-miss data requests in `src/server/common.rs`:
  - `get_stellarhosts_data_cached`
  - `get_exoplanets_data_cached`
- Improved footer readability and content in `src/components/footer.rs`:
  - increased contrast and updated version badge styling
  - added `Developed by imscraping.ninja` link
  - removed link underline
- Removed redundant overview CTA (`Browse Stellar Hosts Catalog`) from `src/components/overview.rs` to keep navigation centered in the floating header.
- Simplified and reorganized GitHub Actions workflows:
  - `tests.yml` now runs only `cargo test`
  - `code-quality.yml` handles formatting, clippy, and typos checks
  - `deploy.yml` now runs as the final stage after successful `Tests` and `Code Quality` workflows for the same `main` commit SHA
- Added typos dictionary config in `.typos.toml` to allow domain/tooling terms
- Resolved strict clippy warnings across the workspace and verified with:
  - `cargo clippy --all-features --workspace -- -D warnings`

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
- Added build-version footer for deployment verification:
  - added a small global footer showing `CARGO_PKG_VERSION` on all routes
  - enables quick confirmation that production is running the expected image/version
  - in `src/components/footer.rs`, `src/components/mod.rs`, `src/app.rs`

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
