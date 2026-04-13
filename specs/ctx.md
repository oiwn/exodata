# Current Context

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
- Stage 1 hub now exists at `/facts`.
- User-facing label is `Insights`, while the route namespace remains `/facts/...`.
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
- Current progress:
  - Insights is now a feature module:
    - `src/components/insights/mod.rs`
    - `overview.rs`
    - `detail.rs`
    - `smallest_exoplanets.rs`
    - `hottest_stellar_hosts.rs`
    - `crowded_systems.rs`
  - Three live routed pages now exist:
    - `/facts/smallest-exoplanets-radius`
    - `/facts/hottest-stellar-hosts`
    - `/facts/systems-with-most-planets`
  - These pages currently use a lightweight shared shell plus page-owned fetch/render flow.
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

- Should insight pages live under a dedicated route namespace such as `/facts/...`?
  - Yes for now: keep `/facts/...` as the route namespace even if the user-facing label is “Insights”
- Should each insight page show exactly 10 results by default, or vary by page?
  - Vary by page; some should be top 5, others top 10 or another explicit limit
- Should result presentation be card-based, table-based, or hybrid depending on the fact type?
  - Depends on fact type; likely each fact family should have its own template
- Which insights should be prioritized first for maximum SEO/GEO value?
  - Scope target is roughly 20-30 new pages, so prioritization and grouping still need to be decided
- Should insights use a heavily generic registry-driven model or a lighter one-file-per-page approach?
  - Decide during Stage 2 after we see the real diversity of fact families
