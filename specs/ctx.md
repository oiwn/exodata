# Current Context

## Bootstrap For Next Session

- Current active work is the Insights feature under `/insights`.
- Branch/worktree has broad uncommitted Insights refactor work; do not assume files are clean.
- Fast orientation files:
  - `src/components/insights/overview.rs` — hub cards and live/planned insight list
  - `src/components/insights/detail.rs` — slug dispatch to live insight components
  - `src/components/insights/common.rs` — shared insight table shell, links, labels, formatting
  - `src/server/functions.rs` — server function module root and shared response types
  - `src/server/functions/insights.rs` — public no-arg Leptos server functions for insight pages
  - `src/server/data/insights.rs` — cached insight query helpers and private reusable query mechanics
  - `src/server/cache.rs` — `InsightCache` and `InsightKind`
- Current live routes:
  - `/insights/smallest-exoplanets-radius`
  - `/insights/largest-exoplanets-radius`
  - `/insights/hottest-stellar-hosts`
  - `/insights/systems-with-most-planets`
  - `/insights/binary-star-systems`
- Route namespace was changed from `/facts` to `/insights`; no redirects are needed because this work is not deployed yet.
- Server function convention:
  - keep `src/server/functions.rs` as the module root
  - put implementation files in `src/server/functions/`
  - public insight server functions should be page-specific and argument-free
- System-level insight convention:
  - use `stellarhosts.sy_name` for system identity
  - use `stellarhosts.sy_pnum` for system-level confirmed planet count
  - use `stellarhosts.sy_snum` for system-level star count
  - include hidden representative `hostname` in system insight payloads so visible `sy_name` can link to `/stellarhosts/{hostname}`
- Host-level insight convention:
  - use `exoplanets.hostname`
  - use `COUNT(DISTINCT pl_name)` when the page is explicitly about planets attached to a host name
- Useful verification commands:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test`
  - `target/debug/exo sql "<SQL>"` for quick parquet-backed validation
- Recent SQL sanity checks:
  - `55 Cnc` has `sy_pnum = 7` at system level.
  - Planet rows are split between `hostname = 55 Cnc` and `hostname = 55 Cnc B`, so do not group `exoplanets.hostname` when answering system-level questions.
  - Binary system query shape: `SELECT sy_name, MIN(sy_pnum), MIN(sy_snum), MIN(sy_dist) FROM stellarhosts WHERE sy_snum = 2 GROUP BY sy_name ORDER BY sy_pnum DESC`.
- Last verified checks after Insights changes:
  - `cargo fmt --check`
  - `cargo check`
  - `cargo test` (`48 passed`)
- Known unrelated/older dirty change:
  - `src/components/stellarhost_detail/planets.rs` links planet cards to `/exoplanets/{planet}`.
- Next likely work:
  - add focused tests for `src/server/data/insights.rs`
  - add more live insight pages from the planned list
  - consider extracting a reusable system-level insight helper/template if another system page is added

## Exoplanet Detail Follow-Up

- Frontend layout refactor is done enough for now:
  - hero visual
  - Earth/Jupiter comparison
  - provenance-style records section
- The remaining important gap is backend/data-shape quality:
  - current payload is still `ExoplanetDetail { pl_name, records, metadata }`
  - summary/provenance UI still derives too much from raw rows on the client
- The durable specification for this work now lives in `specs/exoplanet-detail.md`, especially the `**TODO ASAP**` section.

## Insights Pages For SEO / GEO

### Goal

- Add indexable “Insights” pages aimed at SEO/GEO discovery.
- These pages should expose curated rankings/lists such as:
  - `Smallest exoplanets by radius`
  - `Top 10 exoplanets by radius`
  - `Top 10 hottest exo-suns`
- There should be at least a dozen such pages, so the implementation must scale beyond one-off experiments.

### Product Direction

- There should be an insights landing page or hub.
- Stage 1 hub now exists at `/insights`.
- User-facing label and route namespace are both `Insights` / `/insights/...`.
- The hub now contains both planned cards and live routed insight cards.
- Each insight page should have:
  - a clear title
  - short intro text
  - a structured ranked list or table of results
- The pages should be crawlable and useful as standalone entry points.

### Stages

#### Stage 1

- Build the insights landing page/hub only.
- Use a dummy list of insights.
- Insight cards/links do not need to be clickable yet.
- Primary goal: establish visual direction and page composition.
- Status: completed

#### Stage 2

- Plan and implement the actual fact-page component architecture.
- Facts pages will likely need multiple aggregation/query families, not just one generic `TOP N` pattern.
- Expected fact families:
  - ranked lists such as top-N / bottom-N by a single metric
  - comparison/system pages such as a stellar host and its biggest planets
  - special relationship pages such as equal star-planet pairs or hosts with the most planets
- At this stage we should explicitly decide how generic to be.
- Current leaning:
  - do not over-generalize too early
  - consider one file per insight page if each page needs its own fetch/query/render logic
  - use shared small building blocks only where they clearly reduce duplication
- Likely shared pieces:
  - insights page shell
  - cards for the hub page
  - a few templates for recurring fact families
  - metadata helpers for SEO text
- Possible page ownership model:
  - one component file per insight page
  - each insight component responsible for querying parquet-backed data and rendering its result
  - shared helpers only for repeated query or presentation primitives
- Current implementation direction:
  - do not reuse the main `stellarhosts_table` / `exoplanets_table` pages
  - keep tables/presentations fully on the insight-component side
  - allow lightweight shared shells, but keep page-specific rendering logic local
  - keep public insight server functions page-specific and argument-free
  - keep reusable backend query mechanics private under `src/server/data/insights.rs`
- Current progress:
  - Insights is now a feature module:
    - `src/components/insights/mod.rs`
    - `overview.rs`
    - `detail.rs`
    - `smallest_exoplanets.rs`
    - `hottest_stellar_hosts.rs`
    - `crowded_systems.rs`
    - `binary_systems.rs`
  - Five live routed pages now exist:
    - `/insights/smallest-exoplanets-radius`
    - `/insights/largest-exoplanets-radius`
    - `/insights/hottest-stellar-hosts`
    - `/insights/systems-with-most-planets`
    - `/insights/binary-star-systems`
  - These pages currently use a lightweight shared shell plus page-owned fetch/render flow.
  - Each live insight component calls a dedicated argument-free server function.
  - Insight server functions live under `src/server/functions/insights.rs`.
  - Repeated query mechanics and cache-backed insight data helpers live in `src/server/data/insights.rs`.
  - `src/server/functions.rs` remains the module root; server-function implementation files live under `src/server/functions/`.
  - Insight results are cached through `InsightCache`.
- Important data semantics:
  - `sy_name` is available in `stellarhosts`, not in the current `exoplanets` parquet.
  - NASA defines `sy_name` as the system name, `hostname` as the host/star name, `sy_snum` as the number of gravitationally bound stars in the planetary system, and `sy_pnum` as confirmed planets in the planetary system.
  - Use `sy_name` + `sy_pnum` for system-level insight pages.
  - Use `hostname` + `COUNT(DISTINCT pl_name)` only for host-level insight pages.
  - `/insights/systems-with-most-planets` is system-level and uses `stellarhosts.sy_name` + `stellarhosts.sy_pnum`.
  - `/insights/binary-star-systems` is system-level and filters `stellarhosts.sy_snum == 2`.
  - System-level insight rows include a representative `hostname` in the payload but hide that column in the table; the visible `sy_name` cell links to `/stellarhosts/{hostname}` as a pragmatic bridge until a real system detail route exists.
  - CLI SQL verification showed `55 Cnc` has `sy_pnum = 7` at system level, while planet rows are split between `hostname = 55 Cnc` and `hostname = 55 Cnc B`; do not infer system counts by grouping `exoplanets.hostname`.
- Main question for the next session:
  - does this “one file per insight page + small shared shell” pattern remain the preferred Stage 2 model after a few more examples?

#### Stage 3

- Add tests for insights-page infrastructure and insight logic.
- Prefer testing:
  - query/aggregation helpers
  - fact-page selection logic
  - small presentation helpers
- Avoid brittle page snapshots unless a specific page shape needs protection.

### Likely Building Blocks

- insights index page with reusable link-card component
- optional insights page config registry if it proves useful after Stage 2 decisions
- shared renderers/templates for different fact families, but only where reuse is real
- metadata helpers for per-page SEO text
- server/query helpers for:
  - top-N / bottom-N style result sets
  - comparison/system aggregations
  - special relationship aggregations where needed

### Initial Examples

- smallest exoplanets by radius
- largest exoplanets by radius
- hottest exoplanets by equilibrium temperature
- coldest exoplanets by equilibrium temperature
- nearest stellar hosts
- hottest stellar hosts
- coolest stellar hosts
- most massive stellar hosts
- stellar hosts with the most planets
- binary planetary systems with planets
- planetary systems with the most planets
- most equal star-planet pairs
- stellar hosts with the largest known planet
- compact systems with the shortest orbital periods
- largest planet-to-host size ratios
- hottest planets around the coolest stars
- nearest systems with multiple known planets
- densest small exoplanets
- lowest-density giant exoplanets
- oldest stellar hosts with planets

### Open Questions

- Should insight pages live under a dedicated route namespace such as `/insights/...`?
  - Yes: use `/insights/...` so route names match the user-facing label.
- Should each insight page show exactly 10 results by default, or vary by page?
  - Vary by page; some should be top 5, others top 10 or another explicit limit
- Should result presentation be card-based, table-based, or hybrid depending on the fact type?
  - Depends on fact type; likely each fact family should have its own template
- Which insights should be prioritized first for maximum SEO/GEO value?
  - Scope target is roughly 20-30 new pages, so prioritization and grouping still need to be decided
- Should insights use a heavily generic registry-driven model or a lighter one-file-per-page approach?
  - Decide during Stage 2 after we see the real diversity of fact families
