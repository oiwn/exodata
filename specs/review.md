# Code Review - Table Extensibility and Cleanup (2026-01-30)

## Scope
- Files reviewed: `src/components/exoplanets_table.rs`, `src/components/stellarhosts_table.rs`, `src/components/column_selector.rs`, `src/table/table.rs`, `src/server/common.rs`, `src/server/functions.rs`, `src/server/handlers.rs`, `src/stellarhosts.rs`, `src/common.rs`.
- Goal: reduce duplication, improve maintainability, and prepare for future table filters and new components.

## High-priority refactor candidates
1) Shared table page state + query encoding
- Current state logic, query parsing, and navigation is duplicated in the two table pages.
- Query building drops selected columns when sorting/paging because `build_table_query` only includes page/sort/order. URL state will not round-trip once filters are added.
- Recommendation: introduce a `TableQuery` model and a `use_table_state` helper that handles parse/encode for page/sort/order/columns (and later filters). Use it in both pages.
- Files: `src/components/exoplanets_table.rs`, `src/components/stellarhosts_table.rs`, `src/table/table.rs`.

^^^ this one is good deal

2) Generic table data pipeline on the server
- `get_stellarhosts_data` and `get_exoplanets_data` are near-duplicates with different defaults.
- Recommendation: replace with a `get_table_data(config, params)` or similar helper that handles select/sort/pagination once, and pass a per-table config (default columns, link column, etc.).
- Files: `src/server/common.rs`, `src/server/functions.rs`, `src/server/handlers.rs`.

^^^ we'll do this first

3) Column metadata duplication
- `ColumnMetadata` is duplicated in `src/server/functions.rs` to avoid wasm dependencies. That will become harder to evolve once filters require richer metadata (display labels, filter types, etc.).
- Recommendation: extract a lightweight shared crate (e.g., `exo-types`) or feature-gate exo-core metadata so the type lives in one place.

4) Serialization and dtype coverage
- `dataframe_to_json` only handles a subset of dtypes. New columns with Bool/Date/Time/Utf8/etc. will currently become null.
- The current per-row, per-column access pattern repeats `df.column` lookups and can become costly as filters and column counts grow.
- Recommendation: extend dtype handling and use a row/column iterator once per column or `df.iter_rows()` to avoid repeated lookups.
- File: `src/server/common.rs`.

5) Column labels/tooltips not wired
- `Table` supports optional `column_descriptions`, but the table pages do not pass metadata.
- `format_column_name` hardcodes a few labels and will not scale as new columns are added.
- Recommendation: pass column metadata into `Table` and derive display labels/tooltips from metadata or a per-table map.
- File: `src/table/table.rs` (and table pages).

6) URL encoding for link column
- Link encoding uses ad-hoc string replacements for spaces and '#'.
- Recommendation: use a proper encoder (e.g., `urlencoding` crate) to avoid incorrect URLs for additional characters.
- File: `src/table/table.rs`.

## Medium-priority cleanup and quality
- Extract pagination controls into a reusable component to remove duplication.
- Consider a shared loading and error UI component to keep consistent UX between table pages.
- Column selector says "drag to reorder" but only supports up/down buttons; rename or implement drag-and-drop.
- `SelectedColumnsList` uses `idx` in the key; use the column name as a stable key.
- Files: `src/components/column_selector.rs`, `src/components/exoplanets_table.rs`, `src/components/stellarhosts_table.rs`.

## Tech-debt candidates (likely unused)
- `src/stellarhosts.rs` contains `get_exoplanet_data` but appears unused.
- `src/common.rs` duplicates VOTable helpers from `crates/exo-cli` and has no references.
- Recommendation: confirm usage, then remove or relocate to exo-cli.

## Test gaps
- `src/server/common.rs` tests only cover stellarhosts. Mirror coverage for exoplanets (sort, pagination, column selection).
- Add tests for query encode/decode (page, sort, columns, filters) once the shared state module exists.
- Add tests for `dataframe_to_json` dtype handling.

## Suggested sequencing (pre-filters)
1) Add shared `TableQuery` + `use_table_state` (includes columns in query strings).
2) Extract a generic `TablePage` component and a `PaginationControls` component.
3) Consolidate server `get_*_data` into `get_table_data` with a per-table config.
4) Expand `dataframe_to_json` dtype handling and add tests.
5) Remove or relocate unused modules after confirming no external usage.

## Consideration: Measurement groups (value + err1/err2/lim)
- Many stellarhosts columns appear in sets like `st_teff`, `st_tefferr1`, `st_tefferr2`, `st_tefflim`.
- Proposed approach: keep DataFrame as the source of truth, but introduce a light model that groups related columns for display (e.g., render value with ± error in the table cell).
- Benefits: preserves flexibility of dynamic columns while enabling richer display logic without hardcoding a full Rust struct for all columns.
- Risks: needs a clear mapping rule and confirmation of `err1/err2/lim` semantics; avoid incorrect rendering if a column lacks the companions.

## Open questions
- Should the future filter URL format be `filters=col:value` or `filter[col]=value`?
- Is it acceptable to introduce a small shared crate (e.g., `exo-types`) for `ColumnMetadata` and table config types?
- Do you want column display labels derived from metadata or a separate per-table mapping?
 - Should measurement-group rendering (value ± err) be enabled only for selected columns or as a default for all applicable columns?
