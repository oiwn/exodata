# Current Context

## Current task

[PASTE DETAILED PLAN HERE]

Generic table data pipeline on the server

- `get_stellarhosts_data` and `get_exoplanets_data` are near-duplicates with different defaults.
- Recommendation: replace with a `get_table_data(config, params)` or similar helper that handles select/sort/pagination once, and pass a per-table config (default columns, link column, etc.).
- Files: `src/server/common.rs`, `src/server/functions.rs`, `src/server/handlers.rs`.

### Let's think about it for a while, my thoughts:

- since data are basically in column in-memory database i really do not to know metadata about the table. so we can abstract out actions possible with table.
- we'll need to think that table will need to attach metadata to DataFrame first, some columns are meaningless without context ("err1", "err2", "lim")


#### Consideration: Measurement groups (value + err1/err2/lim)

- Many stellarhosts columns appear in sets like `st_teff`, `st_tefferr1`, `st_tefferr2`, `st_tefflim`.
- Proposed approach: keep DataFrame as the source of truth, but introduce a light model that groups related columns for display (e.g., render value with ± error in the table cell). We shouldn't store data in this model, only use it to quickly fetch data from DataFrame
- Benefits: preserves flexibility of dynamic columns while enabling richer display logic without hardcoding a full Rust struct for all columns.
- Risks: needs a clear mapping rule and confirmation of `err1/err2/lim` semantics; avoid incorrect rendering if a column lacks the companions.
- this fields with "err1", "err2", "lim" should be excluded from the column selection

# Tasks we'll have to do after refactoring (keep in mind)

## Status (Jan 29, 2026)

- VOTable refactor completed: VOTable parsing/loader/conversion moved into `exo-cli`.
- `exo-core` restored table modules (`common`, `overview`, `stellarhosts`, `exoplanets`) and keeps metadata types/TOML helpers.
- Web app and CLI are now separated; shared code lives only in `exo-core`.

## Next Task: Per-Column Filters in Tables

We need filter inputs at the top of each table column (like the screenshot) for both
Stellar Hosts and Exoplanets tables. Filters should update data, totals, and pagination.

### Scope

**Frontend**
- Render a filter row aligned to columns in `src/table/table.rs`.
- Maintain filter state in `src/components/stellarhosts_table.rs` and
  `src/components/exoplanets_table.rs`.
- Include filters in query string/state so filters persist across pagination/sort.

**Backend**
- Extend API query params in `src/server/handlers.rs` to accept filters.
- Apply filters in `src/server/common.rs` before computing `total` and pagination.

### Decisions Needed

- Filter format in URL:
  - Option A: `filters=col1:value,col2:value`
  - Option B: `filter[col1]=value&filter[col2]=value`
- Filter semantics:
  - Strings: case-insensitive contains
  - Numbers: exact match (or optional min/max syntax like `min..max`)

### Acceptance Criteria

- Each visible column has a filter input directly beneath the header.
- Typing a filter updates the table (server-side), with pagination totals reflecting filtered rows.
- Filters persist when sorting or paging.
- Clearing a filter restores the unfiltered dataset.

### Implementation Plan

1. Add filter row to `src/table/table.rs` (inputs aligned to `data.columns`).
2. Add filter state + query encoding/decoding in table pages.
3. Add filter parsing in handlers and apply filters in `server/common.rs`.
4. Add/update tests for filtering in `src/server/common.rs`.

## Refactor Plan (Before Filters)

### 1) Frontend Table State

- Create shared table state module (e.g., `src/table/state.rs`) for:
  - page, sort, order, columns, filters map
  - query-string encode/decode
  - update helpers (`set_filter`, `set_sort`, `set_page`, `clear_filters`)
- Update `stellarhosts_table.rs` + `exoplanets_table.rs` to use the shared state.
- Keep `src/table/table.rs` presentational only.

### 2) Server Query Pipeline

- Add shared helpers in `src/server/common.rs`:
  - `apply_filters(df, filters) -> LazyFrame`
  - `apply_sort(frame, sort_col, order)`
  - `apply_pagination(frame, page, limit)`
- Use the same flow for stellarhosts and exoplanets so totals stay consistent.

### 3) Query Param Model

- Define a single query struct (REST + server functions):
  - `TableQuery { page, limit, sort, order, columns, filters }`
- Implement a single filter parser (URL format chosen once).
- Reuse parser in both `src/server/handlers.rs` and `src/server/functions.rs`.

