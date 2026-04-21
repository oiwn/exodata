# Current Context

## Refactoring — Shitcode Reduction

Goal: eliminate structural repetition without behaviour change.

### 1.1 Rearchitect insights: SQL-driven core module, web/CLI reusable

**Affected files:** `src/server/data/insights.rs`, `src/server/functions/insights.rs`,
`src/components/insights/*.rs`, `src/server/cache.rs`, `crates/exo-core/src/insights.rs`,
`crates/exo-core/src/insights/*.rs`, `crates/exo-cli/src/*`

**Current problem:** adding one insight requires touching 5 layers: data query fn, cached wrapper, server function, component file, router. The logic (what to fetch) is buried in the data layer; the component only owns presentation strings.

**Target architecture:**

`exo-core` owns the reusable insight definitions and execution. The web app renders the result.
The CLI can call the same core functions now or later without duplicating SQL.

```rust
// crates/exo-core/src/insights/smallest_exoplanets.rs
pub const DEF: InsightDef = InsightDef {
    slug: "smallest-exoplanets-radius",
    title: "Smallest Exoplanets By Radius",
    category: "Planetary extremes",
    description: "Tiny confirmed worlds ordered by radius with host-star context.",
    table: InsightTable::Exoplanets,
    limit: 10,
    sql: r#"
        SELECT pl_name, hostname, pl_rade, pl_bmasse, disc_year
        FROM exoplanets
        WHERE default_flag = 1
          AND pl_name IS NOT NULL
          AND pl_name != ''
          AND pl_rade IS NOT NULL
        ORDER BY pl_rade ASC, pl_name ASC
        LIMIT 10
    "#,
};
```

Core registry and API:

```rust
// crates/exo-core/src/insights.rs
pub mod smallest_exoplanets;
pub mod largest_exoplanets;
// ...

pub static INSIGHTS: &[&InsightDef] = &[
    &smallest_exoplanets::DEF,
    &largest_exoplanets::DEF,
    // one line to add a new insight definition
];

pub fn find_insight(slug: &str) -> Option<&'static InsightDef>;
pub fn run_insight(input: InsightInput<'_>, slug: &str) -> anyhow::Result<InsightData>;
pub fn run_insight_def(input: InsightInput<'_>, def: &'static InsightDef) -> anyhow::Result<InsightData>;
pub fn run_all_insights(input: InsightInput<'_>) -> anyhow::Result<Vec<InsightData>>;
```

Core types:

```rust
pub enum InsightTable {
    Exoplanets,
    StellarHosts,
    Both,
}

pub struct InsightDef {
    pub slug: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub table: InsightTable,
    pub sql: &'static str,
    pub limit: usize,
}

pub struct InsightInput<'a> {
    pub stellarhosts: &'a DataFrame,
    pub exoplanets: &'a DataFrame,
}

pub struct InsightData {
    pub slug: &'static str,
    pub columns: Vec<String>,
    pub frame: DataFrame,
}
```

`InsightData` deliberately returns a Polars `DataFrame`, not web `TableData` or JSON. The web
server converts the frame into `TableCacheValue`; the CLI can print the frame directly; future
REST/MCP consumers can convert at their boundary.

The web layer has one generic server function. It validates slug through `exo_core::insights`,
executes the core insight function, converts the returned frame into `TableData`, and caches by
slug:

```rust
#[server(input = GetUrl)]
pub async fn get_insight(slug: String) -> Result<TableData, ServerFnError>
```

Insight components import their core `DEF` and render however they need:

```rust
// src/components/insights/smallest_exoplanets.rs
use exo_core::insights::smallest_exoplanets::DEF;

#[component]
pub fn SmallestExoplanetsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || DEF.slug.to_string(),
        move |slug| async move { get_insight(slug).await },
    );

    view! {
        <InsightListPageShell
            eyebrow=DEF.category
            title=DEF.title
            description=DEF.description
            resource=rows_resource
            empty_label="No planet rows available."
        />
    }
}
```

**What disappears:**
- `InsightKind` enum
- 9 `get_FOO_cached` wrapper functions (`src/server/data/insights.rs:44–159`)
- 9 `#[server]` insight functions (`src/server/functions/insights.rs:12–235`)
- `get_distinct_stellarhosts_data`, `get_distinct_exoplanets_data`, `get_distinct_systems_data`
- `fixed_query`, `get_default_exoplanets_data`, `get_computed_exoplanet_ratio_data`
- All column constant arrays (`SMALLEST_EXOPLANETS_COLUMNS`, etc.)

**What remains / is new:**
- `InsightDef` struct (slug, title, category, description, table, sql, limit)
- `InsightInput` and `InsightData` core structs
- `INSIGHTS` registry in `crates/exo-core/src/insights.rs`
- One SQL executor function in `exo-core` using Polars `SQLContext`
- One web cache wrapper around `exo_core::insights::run_insight`
- Cache keyed by slug (`Cache<String, TableCacheValue>`)
- Cache warm at startup: iterate `exo_core::insights::INSIGHTS`, execute each SQL
- Optional/immediate CLI commands using the same core API:
  `exo insights list`, `exo insights run <slug>`, `exo insights run-all`

**Adding a new insight** = add a core insight definition file, add one line to
`crates/exo-core/src/insights.rs`, and add/render a web component route/card if the insight should
be visible in the UI. No server data wrapper or server function changes.

**Implementation notes:**
- Enable the Polars `sql` feature in `crates/exo-core/Cargo.toml`.
- Keep long page intros, empty states, SEO variants, and special layouts in the web components.
- Keep short reusable metadata (`title`, `category`, `description`, `kind`, `limit`) in a
  lightweight shared crate (`exo-types`), and make Polars-backed `InsightDef` reference that
  metadata.
- Use SQL as the source of truth for insight selection and computed columns. Verify host/system
  dedupe semantics against current output before deleting the old Polars helpers.
- Generate overview cards, detail slug matching, sitemap insight URLs, and startup prewarm from
  `exo_core::insights::INSIGHTS` instead of hardcoded slug lists.

Est. −500 lines in server/data + server/functions layers.

---

### 1.1a Implementation order: CLI insights first

**Status:** core insight registry/execution, `exo insights list`, `exo insights run <slug>`,
`exo insights run-all`, generic web server function, slug-keyed insight cache, startup prewarm, and
registry-backed sitemap insight URLs are implemented. Web insight components currently call
`get_insight(slug)` using page-local slug constants to avoid pulling Polars-backed `exo-core` into
the hydrate bundle.

Start by implementing the reusable core insight module and exposing it through `exo-cli` before
touching the web insight pages. This gives a fast verification loop for every SQL insight and makes
the core API immediately useful for humans, CI, and LLM-driven tooling.

**First vertical slice:**
1. Add `crates/exo-core/src/insights.rs` and `crates/exo-core/src/insights/*.rs`.
2. Add `InsightDef`, `InsightTable`, `InsightInput`, `InsightData`, `INSIGHTS`,
   `find_insight`, `run_insight`, `run_insight_def`, and `run_all_insights`.
3. Enable Polars `sql` in `crates/exo-core/Cargo.toml`.
4. Implement at least one insight SQL definition first, then add the remaining existing insights.
5. Add `exo insights list`, `exo insights run <slug>`, and `exo insights run-all`.
6. Print CLI result frames with `comfy_table` using a minimal-border style.
7. Verify every insight from CLI before wiring the web app to the new core API.

**CLI output requirements:**
- `exo insights list` prints slug, title, category, and row limit.
- `exo insights run <slug>` prints a short heading, description, row count, then the result table.
- `exo insights run-all` prints each insight in registry order; continue running remaining insights
  after a failure, then return an error if any insight failed.
- Use existing data directory defaults: `--data-dir data`.
- Load `stellarhosts.parquet` and `exoplanets.parquet` once per command, then reuse those frames for
  each insight.
- Keep table borders minimal. Prefer a `comfy_table` preset close to the existing
  `table.load_preset("||--+-++|  ")` style used by current CLI sample commands, or a similarly
  sparse preset if that exact preset does not render well for insight output.
- Format scalar values predictably: null as `N/A`, floats with compact precision, integers as-is,
  strings unchanged.
- Do not add web-server cache, Leptos server-function, or component rewiring until the CLI/core path
  is working.

**Suggested command shape:**

```bash
exo insights list
exo insights run smallest-exoplanets-radius
exo insights run-all
exo insights run-all --data-dir data
```

Remaining follow-up after this slice:
- Current phase: add `exo-types` as described in section 1.3.

---

### 1.2 Change `TableResult` to use `TableCacheValue` instead of 4-tuple
**Status:** implemented for table and insight data paths.

**File:** `src/server/data/insights.rs` + all callers

`TableResult = Result<(Vec<Value>, usize, usize, Vec<String>), String>` — the four fields
are already named in `TableCacheValue`. `table_result_from_cache_value` at line 192 is a
pointless destructure that exists only because of the tuple.

**Action:** redefine `TableResult = Result<TableCacheValue, String>`. Remove the destructure
function and all `let (rows, total, total_all, columns) =` spreads. Est. −30 lines.

---

### 1.3 Current phase: split lightweight insight metadata into `exo-types`

**Status:** implemented. `exo-types` now owns hydrate-safe insight metadata, `exo-core`
references that metadata from SQL-backed `InsightDef`s, the frontend overview/detail routing uses
the shared registry, and focused uniqueness/registry-match tests are in place.

**Goal:** share insight slugs/cards/labels between frontend, server, CLI, and future MCP/tooling
without pulling Polars-backed `exo-core` into the hydrate bundle.

**Why:** `exo-core` currently depends on Polars and must remain server/CLI/data-execution only.
Leptos components must not import `exo_core::insights`, even for constants, because that would drag
the data stack toward the frontend build.

**New crate:**

```text
crates/exo-types/
  Cargo.toml
  src/lib.rs
  src/insights.rs
```

`exo-types` must stay lightweight:
- no Polars
- no Leptos
- no Axum / utoipa
- no Tokio
- no CLI formatting
- optional `serde` only if/when needed

Initial type:

```rust
// crates/exo-types/src/insights.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InsightMeta {
    pub slug: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub kind: &'static str,
    pub limit: usize,
}
```

Metadata registry:

```rust
pub mod smallest_exoplanets {
    use super::InsightMeta;

    pub const META: InsightMeta = InsightMeta {
        slug: "smallest-exoplanets-radius",
        title: "Smallest Exoplanets By Radius",
        category: "Planetary extremes",
        description: "Tiny confirmed worlds ordered by radius with host-star context.",
        kind: "Top 10 list",
        limit: 10,
    };
}

pub static INSIGHTS: &[&InsightMeta] = &[
    &smallest_exoplanets::META,
    // ...
];

pub fn find_insight(slug: &str) -> Option<&'static InsightMeta>;
```

Dependency graph after this phase:

```text
exo-types
  shared constants/types only

exo-core
  depends on exo-types
  depends on Polars
  owns SQL execution and parquet/dataframe logic

exo-cli
  depends on exo-core
  can read metadata through exo-core defs or exo-types directly

exoplanets-catalog
  hydrate: depends on exo-types only for insight metadata
  ssr: depends on exo-types + exo-core
```

Change `exo-core::insights::InsightDef` to reference shared metadata instead of duplicating fields:

```rust
use exo_types::insights::InsightMeta;

pub struct InsightDef {
    pub meta: &'static InsightMeta,
    pub table: InsightTable,
    pub sql: &'static str,
}
```

Then per-insight core SQL files become:

```rust
pub const DEF: InsightDef = InsightDef {
    meta: &exo_types::insights::smallest_exoplanets::META,
    table: InsightTable::Exoplanets,
    sql: "...",
};
```

Frontend follow-up:
- Add `exo-types` to the root app hydrate build, but do not add `exo-core`.
- Refactor `src/components/insights/overview.rs` to iterate `exo_types::insights::INSIGHTS`.
- Add a hydrate-safe UI page registry in `src/components/insights/mod.rs` or
  `src/components/insights/registry.rs`:

```rust
pub struct InsightPage {
    pub meta: &'static InsightMeta,
    pub render: fn() -> AnyView,
}
```

- Refactor `detail.rs` to find the page by `meta.slug` and call `render`.
- Replace page-local slug constants with `META.slug`.

Server/CLI follow-up:
- Update all `def.slug`, `def.title`, `def.description`, `def.limit` usages to `def.meta.slug`,
  `def.meta.title`, etc.
- Keep web data fetching through Leptos server functions. Do not add REST/utoipa insight endpoints
  for this refactor.

Tests:
- Add a lightweight test that `exo_types::insights::INSIGHTS` slugs are unique.
- Add an SSR-side test that the UI page registry slugs match `exo_types::insights::INSIGHTS`.
- Keep existing checks:
  - `cargo check --features ssr`
  - `cargo check --no-default-features --features hydrate`
  - `cargo test --package exo-cli`
