# Current Context

## Refactoring — Shitcode Reduction

Goal: eliminate structural repetition without behaviour change.

## Completed

### 1.1 SQL-driven insights

Implemented:
- `exo-core::insights` owns SQL-backed insight definitions and execution.
- `exo-cli` supports `exo insights list`, `exo insights run <slug>`, and `exo insights run-all`.
- Web uses one generic `get_insight(slug)` Leptos server function.
- Insight cache is keyed by slug and prewarmed at startup.
- Sitemap insight URLs are generated from the insight registry.

### 1.2 `TableResult` cleanup

Implemented:
- `TableResult = Result<TableCacheValue, String>`.
- Tuple destructuring helpers/spreads for table payloads are removed.

### 1.3 Hydrate-safe insight metadata

Implemented:
- New lightweight `exo-types` crate owns `InsightMeta`, `INSIGHTS`, and `find_insight`.
- `exo-core::insights::InsightDef` references `&'static InsightMeta` instead of duplicating
  slug/title/category/description/kind/limit.
- Frontend overview cards use `exo_types::insights::INSIGHTS`.
- Frontend detail routing uses a hydrate-safe page registry in
  `src/components/insights/registry.rs`.
- Insight page-local slug constants are replaced with shared `META.slug`.
- Tests cover unique metadata slugs and UI page registry parity.

Validated after 1.3:

```bash
cargo check --features ssr
cargo check --no-default-features --features hydrate
cargo test --package exo-cli
cargo test --package exo-types
cargo test --package exoplanets-catalog --features ssr page_registry_matches_insight_metadata_registry
```

## Current Architecture

Dependency direction:

```text
exo-types
  shared constants/types only; no Polars/Leptos/Axum/Tokio

exo-core
  depends on exo-types + Polars
  owns SQL execution and DataFrame output

exo-cli
  depends on exo-core
  uses the same insight registry/executor as web SSR

exoplanets-catalog
  hydrate: uses exo-types for insight metadata
  ssr: uses exo-types + exo-core for execution/cache/prewarm
```

Adding a visible insight now requires:
- add metadata in `crates/exo-types/src/insights.rs`
- add SQL definition in `crates/exo-core/src/insights/*.rs`
- register both lists in matching order
- add a web page/registry entry only if it should be visible in the UI

## Completed Cleanup

### 1.4 Clean stale specs

**Status:** implemented. `specs/refactoring.md` and this file now describe the current
`exo-types`/`exo-core` architecture and remaining cleanup work.

### 1.5 Fix murky insight table column handling

**Status:** implemented. System insight SQL now returns `sy_name` for display and
`host_link_hostname` as an explicit link helper. The insight table UI hides link-helper columns and
uses `host_link_hostname` for `sy_name` links. CLI insight table output also hides the helper.

File: `src/components/insights/common.rs`

Implemented behavior:
- `render_columns` filters explicit link-helper columns, not columns inferred from `sy_name`.
- `href_for_column("sy_name")` requires `host_link_hostname`; it does not fall back to displayed
  `hostname`.
- Focused tests cover helper-column hiding and system-name link behavior.

### 1.7 Dead-code deletion

**Status:** implemented for the obvious dead code found during this refactor.

Deleted:
- `src/server/common.rs`
- `crates/exo-core/src/tables/exoplanets.rs`
- `crates/exo-core/src/tables/stellarhosts.rs`

Updated:
- `src/server/mod.rs` no longer exports server `common`.
- `crates/exo-core/src/tables.rs` no longer declares deleted table modules.

Validated with:

```bash
cargo check --features ssr
cargo check --no-default-features --features hydrate
cargo test --package exo-cli
```

## Next Steps

### 1.6 Remove duplicate page normalization

Files:
- `src/server/cache.rs`
- `src/server/data/tables.rs`

Current behavior:
- `normalize_table_cache_key` canonicalizes `page == 0` to `1`.
- `get_table_data` also normalizes `page == 0` before pagination.

This is intentionally redundant today because cached and uncached callers can reach
`get_table_data`. Before removing the data-layer guard, verify all direct callers either pass
normalized input or keep the guard and document why both layers normalize.

## Keep In Mind

- Do not infer behavior from fixture values unless the fixture is testing a stable contract.
- Keep `exo-types` lightweight.
- Do not import `exo-core` from hydrate/frontend code.
- Prefer no-behavior-change cleanup unless a spec is updated first.
