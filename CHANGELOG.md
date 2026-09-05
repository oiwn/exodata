# Changelog

## 2026-09-04

- Added the OpenCode GitHub Actions integration, preserving explicit `/oc` and
  `/opencode` commands on issues, pull requests, and inline review comments.
- Added an automatic scanner for same-repository `todos/**` pull requests that
  converts new actionable `TODO`/`FIXME`/`NOTE`/`HACK` comments into labeled
  GitHub issues with stable source fingerprints and semantic deduplication.
- Constrained automatic scans to issue creation with read-only repository
  access, restricted OpenCode tools, per-PR concurrency, and private sessions;
  granted pull-request write access only for the action's required reaction and
  summary comment.
- Verified the workflow end to end: qualifying PRs create appropriate issues
  and PR summaries, repeat scans remain idempotent, and explicit issue commands
  can produce separately reviewable implementation PRs.
- Recorded the successful rollout on GitHub issue #133 and closed it.
- Added a Cargo dependency-audit workflow that runs weekly, supports manual
  dispatch, and checks Rust manifest or lockfile changes on pull requests and
  pushes to `main`; the initial audit surfaced five existing lockfile
  vulnerabilities for follow-up.
- Added server-computed canonical summaries to exoplanet details, deriving
  adopted numeric values from all records with disagreement ranges, counts,
  and provenance while preserving categorical and stable-field evidence.
- Updated the exoplanet summary cards and detail JSON export to use the
  canonical payload, documented the mass fallback and field mappings, removed
  the first-row summary fallback, and verified the change with focused tests,
  formatting, Clippy, and the workspace test suite.

## 2026-08-12

- Added localized explanatory names for stellar spectral classes on the
  overview, including conventional yellow-, orange-, and red-dwarf names.
- Localized planet-size categories, orbital-period units, temperature bands,
  and known discovery methods in English, Simplified Chinese, and Japanese,
  while preserving scientific units, proper names, and unknown source labels.
- Verified the overview localization with focused tests, Rust formatting, and
  manual checks of all supported locales.
- Implemented #126 as a separate pull-request and manually dispatched GitHub
  Actions workflow for the existing Chromium smoke suite.
- Upgraded Playwright to 1.62.1, TypeScript to 7.0.2, and Node typings to the
  Node 24 line, resolving the previous high-severity npm audit findings.
- Added deterministic fixture staging and an `EXO_DATA_DIR` server override so
  E2E runs use small repository fixtures instead of downloading live data.
- Documented the local and CI workflows, including the `fsevents` install-script
  decision and required lazy-route WASM splitting.
- Added the standalone Tailwind CLI required by Cargo Leptos to the E2E runner.
- Verified all six Playwright smoke tests locally, plus TypeScript checking,
  npm audit, Rust formatting, and workflow YAML parsing.
- Completed #115: added a semantic `<main>` landmark to the homepage without
  changing page layout or creating nested landmarks on documentation pages.
- Verified with `cargo check --features ssr` and a manual homepage check.

## 2026-07-21

- Consolidated catalog-table query transitions and successful-result rendering
  while retaining separate stellar-host and exoplanet routes with local data
  resources.
- Added transition coverage and documented shared catalog-table behavior.
- Verified with `cargo clippy --all --workspace`, `cargo test --all --workspace`,
  and manual checks of interactions, browser history, and 404 handling.

## 2026-07-20

- Centralized `ColumnMetadata` in `exo-types`, removing duplicate web/server types and conversion maps while preserving metadata serialization and TOML handling.

## 2026-07-19

- Modernized all ten `src` Rust module entry files from `mod.rs` to the adjacent `name.rs` layout.

## 2026-06-28

- Added a rounded homepage manual section sourced from `docs/index.md` and
  rendered through the shared docs Markdown renderer.
- Added homepage links for stable host/planet examples, JSON/CSV exports, REST
  API docs, MCP docs, CLI docs, and Swagger UI.
- Added hosted MCP setup command boxes with copy buttons for Codex, Claude Code,
  OpenCode, and MCP Inspector, plus a compact CLI/MCP interaction card.
- Completed #119: moved the manual below the detailed homepage statistics and
  linked the hero subtitle to its in-page anchor.
- Removed the local MCP URL from the public MCP connection summary.
- Verified with `cargo clippy --all --workspace`, `cargo test --workspace`, and
  manual browser checks of layout and copy behavior.

## 2026-06-27

- Added detail-page exports for stellar hosts and exoplanets:
  - `.json` suffix downloads return the full detail payload used by the page
  - `.csv` suffix downloads return matching source-table rows
  - export responses include attachment filenames and content types
- Wired the existing detail-page provenance download buttons to real JSON/CSV
  links with tooltips and native download behavior.
- Added MCP `download_detail(entity, name, format)` for read-only JSON/CSV
  detail exports, returning filename, MIME type, content, and URL.
- Documented detail export usage in `docs/api.md`, `docs/mcp.md`, `docs/about.md`,
  and the CLI README MCP summary.
- Captured the implementation plan and TOON deferral in `specs/ctx.md`.
- Verified with manual browser checks, `cargo clippy --all --workspace`, and
  `cargo test --all --workspace`.

## 2026-06-15

- Completed #120: added distinct-planet best-mass distribution bands and the
  five most common stellar spectral classes to the second detailed-statistics
  row on the homepage.
- Added coverage for the canonical aggregation data and homepage statistics
  display.

## 2026-05-25

- Split MCP docs into a dedicated `docs/mcp.md` page (`/docs/mcp`) with
  a "Connecting an Agent" section covering Claude Code, Crush, OpenCode,
  and Codex CLI; `docs/api.md` now links out.
- Added `crates/exo-cli/README.md` and crates.io metadata
  (`readme`/`keywords`/`categories`); bumped `exodata` to `0.1.1`.
- Workspace dependency cleanup: removed dead deps and orphaned
  `examples/` folder; added `[workspace.dependencies]` for `polars`,
  `serde`, `serde_json`, `toml`; dropped vestigial `sqlparser` feature.
- Replaced `anyhow` with `thiserror` in `exo-core` so the library stops
  leaking opaque errors through its public API; `exo-cli` keeps `anyhow`.

## 2026-05-24

- Made the hosted MCP server agent-ready for direct catalog querying:
  - added MCP tool `describe_catalog(table, columns)` so agents can inspect
    column descriptions, units, and data types before writing SQL
  - added MCP tool `query_catalog(sql, limit)` accepting a single read-only
    SQL `SELECT` against `stellarhosts` and `exoplanets`, default 100 rows
    and capped at 1000
  - updated MCP server instructions to point agents at the
    describe-then-query flow
- Extracted SQL validation/execution into a shared `src/server/data/sql.rs`
  helper used by both REST `/rest/query` and MCP `query_catalog`:
  - single table registration site for `stellarhosts` and `exoplanets`
  - shared `validate_sql_select_only` now inspects `SetExpr` and rejects
    `VALUES` and non-SELECT set operations that the prior REST-local check
    let through
  - shared `CatalogSqlError` / `CatalogSchemaError` with per-transport
    status mappers (HTTP vs. MCP)
- Folded REST `/rest/{table}/schema` onto the same shared
  `sql::describe_catalog` helper used by MCP `describe_catalog`, removing
  the last duplicate column-metadata builder
- Added `source_datatype` (the type declared in column metadata TOML) to
  the REST `SchemaResponse.columns[]` shape — additive, OpenAPI-compatible
- Documented MCP connection URLs (local and hosted) and added agent-flow
  examples in `docs/api.md` (basic query, join, aggregate, schema
  discovery); refreshed `specs/cli.md` with the current tool surface
- Added tests for tool listing, `describe_catalog`, `query_catalog`,
  invalid SQL, non-`SELECT`, multiple statements, unknown table, and limit
  capping
- Verified with `cargo clippy --features ssr` and targeted `cargo test
  --features ssr` runs for handlers, sql, and mcp suites

## 2026-05-04

- Expanded focused test coverage for release readiness:
  - added CLI config and output conversion tests
  - added metadata helper and structured data schema tests
  - added server data row conversion, detail lookup, and summary transform tests
  - added table column model and pagination state tests
- Added LLM/agent integration surfaces:
  - added the `exodata` agent skill so coding agents can install project-specific dataset instructions
  - added a hosted MCP server exposing read-only `exodata` tools for dataset-aware reasoning
  - enabled LLM clients to inspect catalog health, list insights, and run curated insights against the dataset
- Improved local CI-style coverage from 34.73% to 47.55% line coverage with the existing `cargo-llvm-cov` workflow settings
- Verified with `cargo fmt --check`, `cargo test --workspace`, and the CI-style `cargo llvm-cov --workspace --summary-only` command

## 2026-04-22

- Consolidated insight definitions around shared SQL execution:
  - moved SQL-backed insight execution into `exo-core::insights`
  - added lightweight shared insight metadata in `exo-types`
  - kept hydrate/frontend insight metadata free of `exo-core` dependencies
  - switched web insight details to one generic `get_insight(slug)` server function
  - kept CLI insight commands on the same registry/executor as web SSR
  - prewarmed all registered insight cache entries at startup
- Simplified table and insight cache payload handling:
  - changed table data operations to return `TableResult = Result<TableCacheValue, String>`
  - removed tuple destructuring helpers for table payloads
  - deleted stale table/server modules that were no longer used
- Fixed insight table link-helper handling:
  - system insights now return display `sy_name` separately from `host_link_hostname`
  - hidden helper columns are filtered explicitly
  - `sy_name` links now require `host_link_hostname` instead of falling back to displayed host text
- Canonicalized table page-zero behavior:
  - `page=0` is treated as page 1 for table data, REST responses, and Leptos server functions
  - browser table routes replace `?page=0` and `?page=1` with canonical URLs that omit `page`
  - sort, order, columns, and filter query parameters are preserved during canonicalization
  - pagination links and table navigation omit `page` for page 1
- Updated `specs/ctx.md` to summarize the completed refactor and current architecture
- Verified with SSR/hydrate checks and focused tests for insight registry parity, canonical table URLs, and REST `page=0` normalization

## 2026-04-13

- Refactored the exoplanet detail page into a feature-owned module and aligned it with the stellar-host detail design family:
  - added `specs/exoplanet-detail.md` to define page architecture, visual direction, and the target backend contract
  - converted `src/components/exoplanet_detail.rs` into `src/components/exoplanet_detail/` with `page.rs`, `hero.rs`, `comparison.rs`, `summary.rs`, `records.rs`, and shared formatting helpers
  - added semantic exoplanet detail styling in `style/components/exoplanet-detail.css` and imported it through `style/tailwind.css`
  - replaced the old emoji-heavy page shell with a planet hero, generated planet visual, and a dedicated Earth/Jupiter radius comparison section
  - corrected comparison scaling to use linear radius proportions while still filling the available comparison space
  - replaced the one-card-per-record records section with a provenance-style summary + dense table layout modeled on stellar-host detail
- Refactored the exoplanets table page to match the current stellar-hosts table architecture:
  - converted `src/components/exoplanets_table.rs` into a feature module with `mod.rs`, `page.rs`, and `sections.rs`
  - extracted page shell, header, loading, error, and pagination UI into smaller exoplanets-specific section components
  - aligned exoplanets page-level styling with semantic feature classes in `style/components/exoplanets-table.css`
  - imported the exoplanets feature stylesheet through `style/tailwind.css`
- Expanded shared table-page infrastructure in `src/table/`:
  - added `TablePaginationState` for repeated pagination view state
  - added `TableQuerySignals` to group shared query-related signals and query snapshot helpers
  - applied the shared pagination/query state abstractions across both table pages
- Verified the refactor with `cargo fmt`, `cargo check`, `cargo clippy --all-features --workspace -- -D warnings`, and manual browser validation

## 2026-04-12

- Refactored the stellar hosts table page into smaller, feature-owned pieces:
  - introduced shared table query state + navigation helpers in `src/table/query_navigation.rs`
  - moved both table pages and pagination links onto `TableQueryState`
  - added focused query-navigation tests
  - split `stellarhosts_table` into a dedicated module with `page.rs` and `sections.rs`
  - extracted page shell, header, loading, error, and pagination UI into smaller components
  - centralized stellar-hosts table page transitions through a single route-specific navigation path
- Added feature-scoped semantic styling for the stellar hosts table page:
  - created `style/components/stellarhosts-table.css`
  - imported it from the active Tailwind entrypoint `style/tailwind.css`
  - moved page-level shell/header/loading/error/pagination styling out of inline Rust class strings
- Verified the refactor with `cargo fmt`, `cargo check`, targeted query-navigation tests, and manual browser validation

## 2026-04-06

- Fixed overview entity totals and breakdown semantics:
  - changed overview stellar host / exoplanet totals to count distinct `hostname` and `pl_name` instead of raw row counts (`crates/exo-core/src/tables/overview.rs`, `src/main.rs`)
  - replaced the exoplanet overview card subtitle with `Distinct planets in the catalog` (`src/components/overview.rs`)
  - reworked overview discovery-method and radius-classification sections to use one canonical value per planet instead of counting all records
  - added overview sections for distinct planets by earliest discovery year and canonical orbital-period bucket
  - added focused overview aggregation tests covering distinct totals, canonical method selection, canonical radius selection, earliest discovery year, and orbital-period bucketing
- Improved global shell/UI polish:
  - added GitHub repository link to the navbar on desktop and mobile (`src/components/navbar.rs`)
  - added compile-time build timestamp to the footer via `build.rs` and rendered it as `Updated` next to the version badge (`build.rs`, `src/components/footer.rs`)
  - replaced the default plain 404 output with a branded not-found page matching the site visual style (`src/error_template.rs`)
- Added agent guidance clarifying that test fixtures are sample material for tests and not source-of-truth dataset values (`AGENTS.md`)

## 2026-04-05

- Added baseline SEO infrastructure for crawlability and metadata:
  - added static `robots.txt` in `public/robots.txt` with open crawling and sitemap reference
  - added startup-built, in-memory cached `GET /sitemap.xml` served by Axum (`src/main.rs`, `src/server/handlers.rs`, `src/server/mod.rs`)
  - sitemap includes canonical static pages plus distinct stellar host and exoplanet detail URLs
  - added sitemap route test coverage in `src/server/tests.rs`
- Added page-level SSR-friendly metadata across the app:
  - introduced shared metadata helpers for titles, descriptions, canonical URLs, and percent encoding/decoding (`src/metadata_helpers.rs`)
  - added per-page `title`, `meta description`, and canonical tags for overview, about, stellar hosts table, exoplanets table, stellar host detail, and exoplanet detail pages
  - detail-page metadata now derives from the same SSR resource data used to render page content
  - removed duplicate global description tag from the app shell so each page emits a single description
- Replaced manual route param decoding for detail pages with proper percent decoding (`percent-encoding` in `Cargo.toml`)
- Added structured data / JSON-LD for SEO without a dedicated schema crate:
  - introduced `src/structured_data.rs` to build `serde_json` schema payloads and render SSR `application/ld+json` scripts
  - added `WebSite` schema to `/`, `CollectionPage` schema to `/stellarhosts` and `/exoplanets`, and `Dataset` schema to stellar host and exoplanet detail pages
- Fixed detail-page hydration warnings caused by reading SSR resources in head tags outside suspense:
  - moved resource-backed `Title`, meta description, and JSON-LD emission into the successful `<Suspense/>` branch on detail pages
  - switched detail-page canonical href generation to non-reactive values to avoid unnecessary reactive read warnings during hydrate

## 2026-04-03

- Reworked stellar host detail into a canonical host profile instead of a `first row wins` record view:
  - added per-`hostname` canonicalization for identity, stable system values, median-based numeric summaries, categorical summaries, and provenance (`src/server/stellarhost_canonical.rs`, `src/server/common.rs`, `src/server/functions.rs`)
  - added host-detail caching in server state (`src/server/cache.rs`, `src/server/handlers.rs`, `src/main.rs`)
  - redesigned the detail page around hero, canonical summary, planets, and provenance sections
  - converted provenance reference markup into real outbound links with `nofollow`
- Refactored stellar host detail UI into a dedicated submodule:
  - route container in `src/components/stellarhost_detail/page.rs`
  - section files for hero, star visual, summary, planets, provenance, and shared formatting helpers
- Added illustrative star-color rendering driven by canonical `st_teff`:
  - introduced curated temperature-to-color mapping and derived hero visual tokens (`src/components/stellarhost_detail/star_color.rs`)
  - wired hero star rendering to canonical temperature with neutral fallback when missing

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
