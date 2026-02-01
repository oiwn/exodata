# Current Context

## Current task

### Done (this iteration)
- ✅ Measurement groups model (names only); err/lim hidden in selector.
- ✅ Stellarhosts renders err1/err2 mini values; headers use metadata tooltips.
- ✅ Fetch includes err/lim companions; sort remains on base columns.
- ✅ Measurement rendering rolled into Exoplanets.
- ✅ Server table pipeline unified via `get_table_data`.
- ✅ First-column text filter (server-side) with blur/enter commit; query param `filter`.

### Next
- Finalize `lim` indication styling for measurement cells.

# Tasks to keep in mind (post-refactor)

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
