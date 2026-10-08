# Changelog

## 2026-10-08 — e2e discovery year heading

- Fix the date-sort E2E test to locate the abbreviated Discovery year heading while preserving scientific sort keys.
- Task `0015-e2e-discovery-year-heading`; source: https://github.com/oiwn/exodata/pull/158

## 2026-10-08 — catalog default value abbreviations

- Shorten discovery-method and localized mass headings and wide discovery-method values, retain immediate explanatory tooltips, and align the compact filter input with the first column.
- Task `0013-catalog-default-value-abbreviations`

## 2026-10-08 — seo audit fixes

- Fix detail trailing-slash/malformed-path handling and URL consistency, remove duplicate description/lang tags, and emit record metadata only for successful profiles.
- Task `0012-seo-audit-fixes`; source: https://github.com/oiwn/exodata/issues/159

## 2026-10-08 — homepage counter links

- Make the homepage Stellar Systems and Exoplanets counter cards clickable links to their locale-aware catalogs, with visible keyboard focus.
- Task `0011-homepage-counter-links`; source: https://github.com/oiwn/exodata/issues/156

## 2026-10-08 — catalog detail error pages

- Render branded 404 pages for missing stellar hosts and exoplanets and branded 500 pages for internal loading failures, with correct SSR status and no raw server error text.
- Task `0010-catalog-detail-error-pages`; source: https://github.com/oiwn/exodata/issues/155

## 2026-10-08 — homepage planet title glyph

- Replace the homepage galaxy glyph with a decorative ringed planet outside the gradient title text in all locales.
- Task `0009-homepage-planet-title-glyph`; source: https://github.com/oiwn/exodata/issues/154

## 2026-10-08 — exoplanet table localized headings

- Localize the eight default exoplanet display headings while preserving scientific column identifiers and custom headings. Include translated catalog controls, locale-preserving navigation, and single-line table text.
- Task `0008-exoplanet-table-localized-headings`; source: https://github.com/oiwn/exodata/issues/153

## 2026-10-08 — stellarhost radius caption

- Use parentheses for the stellar radius comparison caption's scaling remark.
- Task `0007-stellarhost-radius-caption`; source: https://github.com/oiwn/exodata/issues/152

## 2026-10-08 — stellarhost radius comparison layout

- Show the stellar radius comparison in one shared dark panel by removing the individual star card decoration.
- Task `0006-stellarhost-radius-comparison-layout`; source: https://github.com/oiwn/exodata/issues/151

## 2026-10-08 — stellarhost source text contrast

- Make stellar-host summary source labels and the empty-source message readable against the dark detail background.
- Task `0005-stellarhost-source-text-contrast`; source: https://github.com/oiwn/exodata/issues/150

## 2026-10-08 — detail color label contrast

- Improve stellar-host color badge contrast with black text and an opaque pale temperature-derived background.
- Task `0004-detail-color-label-contrast`; source: https://github.com/oiwn/exodata/issues/149

## 2026-10-08 — homepage mcp description

- Replace the homepage MCP manual with a brief localized introduction and setup link, and improve homepage discovery metadata.
- Task `0003-homepage-mcp-description`; source: https://github.com/oiwn/exodata/issues/148

## 2026-10-08 — exoplanets mcp seo research

- Research exoplanet MCP discovery and implement specific guide metadata, verified setup, data provenance answers, and MCP sitemap coverage.
- Task `0002-exoplanets-mcp-seo-research`; source: https://github.com/oiwn/exodata/issues/146

## 2026-10-08 — exoplanets default date sort

- Default the exoplanets website to newest update dates first, with an Updated column, undated records last, stable ties, and preserved sort and column URL state.
- Task `0001-exoplanets-default-date-sort`; source: https://github.com/oiwn/exodata/issues/143

## 2026-10-08 — exoplanet host star links

- Link Host star values in the exoplanet table to locale-aware stellarhost profiles while preserving planet links and plain missing-value cells.
- Task `0014-exoplanet-host-star-links`

## 2026-09-17

- Generated the full v10 stellar-host description catalog: 4,768/4,768 systems, zero failures, 7,091,608 recorded tokens (stats snapshot at `content/stats/v10.json`). Plain measurement values throughout: 6 ordinary `about` uses remain corpus-wide, zero Earth-year comparisons, zero raw `M sin i`.
- Algorithmic-bolding fix: thousands-grouped numbers now bold whole (`**1,410 light-years**`) instead of splitting at the comma, and `dev descriptions normalize` repairs previously split spans in stored articles (148 instances across 110 files cleaned).
- Post-batch copy-edits: repaired `an mean density` grammar in 8 articles; moved the shared guide to `content/stellarhost_guide.toml` (specs already described it at `content/` root).
- Descriptions ship to production via a mounted volume instead of the Docker image: new `just ansible-upload-descriptions` rsyncs only `description.md` files to the droplet, `deploy.yml` mounts them at `/app/content/systems:ro`, and the runtime `COPY` of gitignored content was removed so GitHub Actions builds succeed from a clean checkout. Prose updates no longer require an image rebuild.

## 2026-09-12

- Split the descriptions pipeline into the new `exodata-prose` crate (`crates/exo-prose`); exo-cli keeps CLI wiring only. `dataframe_to_json` moved to `exo_core::json`.
- Built the `dev descriptions analyze` suite: `text`, `summary`, `ngrams --openers`, `templates` (number/unit-masked), `tropes`, `metadata`, `anomalies` (outliers, cross-system duplicate sentences, unbolded measurements), `compare --baseline-dir`, `report --output-path`, and `snapshot --output-path` (machine-readable stats under gitignored `content/stats/`; pristine pass-1 recorded as `content/stats/pass1.json`).
- De-monotonization round 1: prepare gates the Earth-year comparison to year extremes (≤10 days / ≥3 years); drafter prompt requires varied inventory phrasing, at most one discovery-method explanation per article, planet grouping, selective comparisons, folded rankings, varied rhythm, and no recap endings; request wording no longer teaches "associated".
- Rebuilt the pipeline to draft-style, fingerprint v5: the JSON critic was removed (13.5% of prompt tokens, mostly false-positive findings); the editor became a compact draft-only style pass under `content/stellarhost_style_prompt.txt`; metadata records draft/style stages.
- Deterministic narrative gates with gates-as-critic flow: the draft retries only on fact gates; all narrative violations (opener share 40%/8, sentence-length spread ≥2.0, numeric density ≤2.5, banned machine phrasings, number+unit reuse) are forwarded to the style pass as an explicit defect list; only style exhaustion fails the system.
- Extended the mechanical post-pass: em dashes/double hyphens become single hyphens, repeated `about` collapses, planet names and spectral labels are auto-bolded outside strong spans; the corresponding gates are backstops. `times Earth` repair consumes typographic possessives.
- Baseline-15 calibration across four runs reached 15/15 with all failure classes resolved (year anchors licensed-only, TRAPPIST-1 openers fixed via forwarded defects, hostname/spectral/365 reuse false positives exempt, Kepler-11 value-collision class fixed by number+unit pair keying). Call economics: ~2 calls/system, ~4-6k tokens/system vs 8.4k pass 1.

## 2026-09-10

- Completed the full-catalog prose generation for #116: all 4,768 preparable systems described (~39.9M recorded tokens including retry passes; one permanent skip, 2MASS J11011926-7732383, with no usable stellar-host summary row). The first pass produced 4,494 systems; the remaining 274 gate failures cleared across `--failed` retry passes.
- Algorithmic formatting: stage outputs now pass a normalizer that bolds measurement phrases (number+unit, full `times Earth's` forms, spectral label) outside strong spans and repairs the density possessive before the gates; the bolding gate is a backstop only. Bolding failures were 67% of first-pass gate failures.
- Added `dev descriptions normalize` to re-apply the normalizers to stored descriptions (retro-fix rewrote 258 files; zero possessive slips remain).
- Per-system commands (`generate-batch`, `status`, `prepare`, `normalize`) default to a compact single-line `lines` output; table/json/csv remain available via `--output`. Batch outcome rows now include attempts.
- Prose pipeline for this pass: draft → JSON-mode critic → editor with deterministic gates (Markdown structure, licensing-aware banned phrases, numeric allowlist, fact preservation), fingerprint v4.
- Curated licensed facts: per-planet `circumbinary` (NASA `cb_flag`), star `host_kind = "pulsar"`, `age_class = "very young"` (<0.1 Gyr), and "No orbital period is reported" facts, each with guide entries; label bans lift only with their facts.
- Preparation scale-up: `Catalog` loads source Parquet files once; `prepare --all` enumerates all hostnames, `--dry-run` validates without writing; 4,769-system dry run classified failures and diagnostics.
- Generation operations: `generate-batch --failed` retries failed systems over matching fingerprints; new `dev descriptions status` summarizes per-system state, usage, and failures with aggregate totals.
- Per-system manual notes channel: tracked optional `notes.toml` (`facts` merge into publishable comparisons, `guidance` into silent constraints); fingerprint covers the merge, so edited notes regenerate.
- Content layout for catalog scale: tracked set slimmed to `description.md` + `notes.toml`; request/metadata/evidence/draft/fail files ignored.
- Tooling alignment: repo alias `cargo lx` (clippy --workspace --all-targets --all-features) shared by prek hook, Justfile, and rust-analyzer; prek test hook fixed to `--workspace` (previously ran only the root package's tests); ~27 clippy warnings fixed.
- Verified each round with the workspace suites (62 CLI lib tests, integration, 137 web) and live baseline regenerations.

## 2026-09-06

- Merged PR #142: added a single app-wide `<main>` landmark around routed content, including error pages, and replaced page-level landmarks in the overview, about, and docs components to avoid duplicates. PR CI passed formatting, Clippy, tests, coverage, typos, and Playwright smoke checks.
- Archived the completed Rust dependency upgrade and hardening task: updated canonical dependency requirements and the lockfile, adapted source code to upgraded APIs, and enforced locked resolution across build and CI entry points.
- Added Cargo Audit policy with scoped exceptions for `RUSTSEC-2026-0194` (trusted offline VOTable input) and `RUSTSEC-2026-0195` (unused Polars cloud XML paths), while keeping informational advisories visible.
- Retained Serde at exactly `1.0.228` for VOTable `0.7.0` compatibility; documented the constraint in the technical overview. Updated `h2` to `0.4.19` and `chacha20` to `0.10.2`.
- Dependency-task verification recorded before archival: locked compile, CI-scope Clippy, 175 workspace tests, release cargo-leptos build, coverage, six Playwright smoke tests, formatting, and workflow validation passed. Cargo Audit passed with accepted Bincode, Paste, and proc-macro-error2 informational warnings.

## 2026-09-04

- Added the OpenCode GitHub Actions integration, preserving explicit `/oc` and `/opencode` commands on issues, pull requests, and inline review comments.
- Added an automatic scanner for same-repository `todos/**` pull requests that converts new actionable `TODO`/`FIXME`/`NOTE`/`HACK` comments into labeled GitHub issues with stable source fingerprints and semantic deduplication.
- Constrained automatic scans to issue creation with read-only repository access, restricted OpenCode tools, per-PR concurrency, and private sessions; granted pull-request write access only for the action's required reaction and summary comment.
- Verified the workflow end to end: qualifying PRs create appropriate issues and PR summaries, repeat scans remain idempotent, and explicit issue commands can produce separately reviewable implementation PRs.
- Recorded the successful rollout on GitHub issue #133 and closed it.
- Added a Cargo dependency-audit workflow that runs weekly, supports manual dispatch, and checks Rust manifest or lockfile changes on pull requests and pushes to `main`; the initial audit surfaced five existing lockfile vulnerabilities for follow-up.
- Added server-computed canonical summaries to exoplanet details, deriving adopted numeric values from all records with disagreement ranges, counts, and provenance while preserving categorical and stable-field evidence.
- Updated the exoplanet summary cards and detail JSON export to use the canonical payload, documented the mass fallback and field mappings, removed the first-row summary fallback, and verified the change with focused tests, formatting, Clippy, and the workspace test suite.

## 2026-08-12

- Added localized explanatory names for stellar spectral classes on the overview, including conventional yellow-, orange-, and red-dwarf names.
- Localized planet-size categories, orbital-period units, temperature bands, and known discovery methods in English, Simplified Chinese, and Japanese, while preserving scientific units, proper names, and unknown source labels.
- Verified the overview localization with focused tests, Rust formatting, and manual checks of all supported locales.
- Implemented #126 as a separate pull-request and manually dispatched GitHub Actions workflow for the existing Chromium smoke suite.
- Upgraded Playwright to 1.62.1, TypeScript to 7.0.2, and Node typings to the Node 24 line, resolving the previous high-severity npm audit findings.
- Added deterministic fixture staging and an `EXO_DATA_DIR` server override so E2E runs use small repository fixtures instead of downloading live data.
- Documented the local and CI workflows, including the `fsevents` install-script decision and required lazy-route WASM splitting.
- Added the standalone Tailwind CLI required by Cargo Leptos to the E2E runner.
- Verified all six Playwright smoke tests locally, plus TypeScript checking, npm audit, Rust formatting, and workflow YAML parsing.
- Completed #115: added a semantic `<main>` landmark to the homepage without changing page layout or creating nested landmarks on documentation pages.
- Verified with `cargo check --features ssr` and a manual homepage check.

## 2026-07-21

- Consolidated catalog-table query transitions and successful-result rendering while retaining separate stellar-host and exoplanet routes with local data resources.
- Added transition coverage and documented shared catalog-table behavior.
- Verified with `cargo clippy --all --workspace`, `cargo test --all --workspace`, and manual checks of interactions, browser history, and 404 handling.

## 2026-07-20

- Centralized `ColumnMetadata` in `exo-types`, removing duplicate web/server types and conversion maps while preserving metadata serialization and TOML handling.

## 2026-07-19

- Modernized all ten `src` Rust module entry files from `mod.rs` to the adjacent `name.rs` layout.

## 2026-06-28

- Added a rounded homepage manual section sourced from `docs/index.md` and rendered through the shared docs Markdown renderer.
- Added homepage links for stable host/planet examples, JSON/CSV exports, REST API docs, MCP docs, CLI docs, and Swagger UI.
- Added hosted MCP setup command boxes with copy buttons for Codex, Claude Code, OpenCode, and MCP Inspector, plus a compact CLI/MCP interaction card.
- Completed #119: moved the manual below the detailed homepage statistics and linked the hero subtitle to its in-page anchor.
- Removed the local MCP URL from the public MCP connection summary.
- Verified with `cargo clippy --all --workspace`, `cargo test --workspace`, and manual browser checks of layout and copy behavior.

## 2026-06-27

- Added detail-page exports for stellar hosts and exoplanets:
  - `.json` suffix downloads return the full detail payload used by the page
  - `.csv` suffix downloads return matching source-table rows
  - export responses include attachment filenames and content types
- Wired the existing detail-page provenance download buttons to real JSON/CSV links with tooltips and native download behavior.
- Added MCP `download_detail(entity, name, format)` for read-only JSON/CSV detail exports, returning filename, MIME type, content, and URL.
- Documented detail export usage in `docs/api.md`, `docs/mcp.md`, `docs/about.md`, and the CLI README MCP summary.
- Captured the implementation plan and TOON deferral in `specs/ctx.md`.
- Verified with manual browser checks, `cargo clippy --all --workspace`, and `cargo test --all --workspace`.

## 2026-06-15

- Completed #120: added distinct-planet best-mass distribution bands and the five most common stellar spectral classes to the second detailed-statistics row on the homepage.
- Added coverage for the canonical aggregation data and homepage statistics display.

## 2026-05-25

- Published dedicated MCP setup documentation and CLI README/crates.io metadata; released exodata 0.1.1.
- Consolidated workspace dependencies, removed obsolete dependencies/examples, and replaced exo-core's anyhow errors with thiserror.

## 2026-05-24

- Added MCP describe_catalog and query_catalog for schema discovery and read-only SELECT queries (default 100 rows, capped at 1000).
- Shared SQL validation/execution and schema discovery between REST and MCP; rejected VALUES/non-SELECT set operations and added source_datatype to REST schema metadata.
- Updated agent connection/query documentation and verified tool listing, SQL/schema errors, and limit handling with focused tests and Clippy.

## 2026-05-04

- Expanded CLI, metadata/schema, server data, and table-state tests; added the public exodata agent skill and hosted read-only MCP tools. CI-style line coverage increased from 34.73% to 47.55%; formatting, workspace tests, and coverage checks passed.

## 2026-04-22

- Unified SQL-backed insights across core, web, and CLI with shared metadata, generic detail loading, startup prewarming, and simpler cached payloads.
- Separated insight display names from explicit host-link columns and hid helper columns. Normalized browser page 0/1 URLs to omit page while preserving query state; backend page 0 uses page 1. Focused SSR/hydration, registry, REST, and URL checks passed.

## 2026-04-13

- Refactored planet details and the exoplanet table into feature-owned modules with semantic styles, shared pagination/query signals, and provenance summaries/tables. Added measured planet visuals and linear Earth/Jupiter radius comparisons; formatting, compile, Clippy, and manual checks passed.

## 2026-04-12

- Extracted shared TableQueryState/navigation and split stellarhost table rendering into feature-owned sections with semantic CSS. Verified transitions with focused tests, compile/format checks, and manual navigation.

## 2026-04-06

- Corrected distinct-host/planet totals and canonical overview distributions; added earliest-discovery-year and period buckets with aggregation tests. Added the navbar GitHub link, build timestamp, branded 404, and guidance that fixtures are samples rather than dataset truth.

## 2026-04-05

- Added open robots.txt crawling, cached static/detail sitemaps, shared metadata/URL helpers, percent-decoded detail routes, and WebSite/CollectionPage/Dataset JSON-LD.
- Moved resource-backed detail metadata into successful Suspense branches to address hydration warnings and removed the duplicate global description.

## 2026-04-03

- Introduced cached canonical stellarhost profiles with identity, initial median-based numeric summaries, categorical evidence, related planets, and provenance links. Split detail UI into sections and added temperature-derived star colors with a missing-value fallback.

## 2026-03-29

- Added configurable server tracing, lazy-route WASM splitting (initial load reduced from 1.2MB to 535KB), and cargo-llvm-cov/Codecov reporting for main pushes, PRs, and manual runs.

## 2026-03-11

- Fixed the pre-hydration interaction gap (#26) and single-vCPU SSR starvation using an interaction overlay and four Tokio workers. Switched table routes to out-of-order streaming at that time and upgraded Polars 0.52 → 0.53 with API adaptations.

## 2026-02-18 (Update 2)

- Restored Async table SSR and moved test/quality checks to PR/manual triggers; Docker builds run on main pushes/manual dispatch with Cargo.toml version-bump gating.

## 2026-02-18

- Moved table cache misses to spawn_blocking, improved footer readability, removed a redundant homepage CTA, separated test/quality/deploy workflows, added the typos dictionary, and resolved strict workspace Clippy warnings.

## 2026-02-13

- Delivered table metadata once through the global hydration store (#15), removed metadata from table responses/caches, and fixed metadata-script placement. Added sequential Chromium SSR/navigation smoke tests with readiness polling and documented their workflow; added the build-version footer.

## 2026-02-12

- Added overview/table caches and cache lifecycle tests, enabled UUID JS support for WASM builds, avoided no-op metadata writes that destabilized SSR, and made ColumnMetadata comparable.
