# Refactoring Notes

This file tracks remaining cleanup after the insight refactor. Use `specs/ctx.md` for active turn
context.

## Completed

### SQL-backed insights

The old repeated insight stack has been replaced.

Implemented:
- `exo-types` owns lightweight shared insight metadata.
- `exo-core::insights` owns SQL-backed Polars execution.
- `exo-cli` can list and run insights through the shared core executor.
- Web SSR uses one generic `get_insight(slug)` server function.
- Insight cache is keyed by slug and prewarmed at startup.
- Sitemap insight URLs are generated from the registry.
- Web hydrate uses `exo-types`, not `exo-core`.
- Insight overview/detail routing is registry-backed.

### Table result shape

Implemented:
- `TableResult = Result<TableCacheValue, String>`.
- Old unnamed tuple payload plumbing is removed.

## Current Dependency Rule

```text
exo-types
  shared metadata/types only

exo-core
  exo-types + Polars execution

exo-cli
  exo-core

exoplanets-catalog hydrate
  exo-types only for insight metadata

exoplanets-catalog ssr
  exo-types + exo-core
```

Do not import `exo-core` from hydrate/frontend code.

## Completed Cleanup

### Insight system-table display/link split

Implemented:
- system insight SQL returns `sy_name` for display and `host_link_hostname` for links
- web insight tables hide explicit link-helper columns
- `sy_name` links require `host_link_hostname`
- CLI insight table output hides the link helper

### Deleted/dead-code cleanup

Deleted:
- `src/server/common.rs`
- `crates/exo-core/src/tables/exoplanets.rs`
- `crates/exo-core/src/tables/stellarhosts.rs`

Updated:
- `src/server/mod.rs`
- `crates/exo-core/src/tables.rs`

Validated with:

```bash
cargo check --features ssr
cargo check --no-default-features --features hydrate
cargo test --package exo-cli
```

## Remaining Cleanup

### Page normalization ownership

Files:
- `src/server/cache.rs`
- `src/server/data/tables.rs`

Current behavior:
- `normalize_table_cache_key` canonicalizes `page == 0` to `1`.
- `get_table_data` also canonicalizes `page == 0` before pagination.

Do not remove the data-layer guard blindly. `get_table_data` can be called directly. First inspect
callers, then either:
- keep both guards and document the defensive behavior, or
- ensure every caller passes normalized input and remove the data-layer guard.

## Guardrails

- Keep `exo-types` lightweight: no Polars, Leptos, Axum, Tokio, or CLI formatting.
- Do not duplicate insight metadata in web components.
- Prefer no-behavior-change cleanup unless the relevant spec is updated first.
- Do not infer behavior from fixture values unless the fixture is testing a stable contract.
