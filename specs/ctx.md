# Current Context

## Refactoring — Shitcode Reduction

Goal: eliminate structural repetition without behaviour change.

### 1.1 Rearchitect insights: SQL-driven, component-owned, registry-aggregated

**Affected files:** `src/server/data/insights.rs`, `src/server/functions/insights.rs`,
`src/components/insights/*.rs`, `src/server/cache.rs`

**Current problem:** adding one insight requires touching 5 layers: data query fn, cached wrapper, server function, component file, router. The logic (what to fetch) is buried in the data layer; the component only owns presentation strings.

**Target architecture:**

Each insight component owns its SQL and exports a config struct:

```rust
// src/components/insights/smallest_exoplanets.rs
pub const DEF: InsightDef = InsightDef {
    slug:  "smallest-exoplanets-radius",
    table: InsightTable::Exoplanets,
    sql:   "SELECT pl_name, hostname, pl_rade, pl_bmasse, disc_year
            FROM exoplanets WHERE default_flag = 1
            ORDER BY pl_rade ASC LIMIT 10",
};

#[component]
pub fn SmallestExoplanetsPage() -> impl IntoView {
    // fully handmade — custom layout, labels, whatever it needs
}
```

Registry in `mod.rs` is the aggregation point only — one line per insight:

```rust
// src/components/insights/mod.rs
pub static INSIGHTS: &[&InsightDef] = &[
    &smallest_exoplanets::DEF,
    &largest_exoplanets::DEF,
    // one line to add a new insight
];
```

One generic server function validates slug against `INSIGHTS`, executes SQL via Polars
`SQLContext`, caches by slug:

```rust
#[server(input = GetUrl)]
pub async fn get_insight(slug: String) -> Result<TableData, ServerFnError>
```

**What disappears:**
- `InsightKind` enum
- 9 `get_FOO_cached` wrapper functions (`src/server/data/insights.rs:44–159`)
- 9 `#[server]` insight functions (`src/server/functions/insights.rs:12–235`)
- `get_distinct_stellarhosts_data`, `get_distinct_exoplanets_data`, `get_distinct_systems_data`
- `fixed_query`, `get_default_exoplanets_data`, `get_computed_exoplanet_ratio_data`
- All column constant arrays (`SMALLEST_EXOPLANETS_COLUMNS`, etc.)

**What remains / is new:**
- `InsightDef` struct (slug, table, sql)
- `INSIGHTS` registry in `mod.rs`
- One SQL executor function (replaces all the Polars pipeline code)
- Cache keyed by slug (`Cache<String, TableCacheValue>`)
- Cache warm at startup: iterate `INSIGHTS`, execute each SQL
- CLI command: `exo-cli insights run <slug>` / `exo-cli insights run-all`

**Adding a new insight** = write component file, export `DEF`, add one line to `mod.rs`.
No other files touched.

Est. −500 lines in server/data + server/functions layers.

---

### 1.2 Change `TableResult` to use `TableCacheValue` instead of 4-tuple
**File:** `src/server/data/insights.rs` + all callers

`TableResult = Result<(Vec<Value>, usize, usize, Vec<String>), String>` — the four fields
are already named in `TableCacheValue`. `table_result_from_cache_value` at line 192 is a
pointless destructure that exists only because of the tuple.

**Action:** redefine `TableResult = Result<TableCacheValue, String>`. Remove the destructure
function and all `let (rows, total, total_all, columns) =` spreads. Est. −30 lines.


