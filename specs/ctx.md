# Current Context

## Current task

### Planet detail page (agreed plan)
- Route: add `/exoplanets/:pl_name` to mirror stellar host detail routing.
- Key: use `pl_name` in the URL; handle duplicates by listing all matching rows on the same page.
- Component: `ExoplanetDetailPage` modeled after `StellarHostDetailPage`.
- Data: load all columns + metadata; render full record(s) with err/lim formatting.
- Links: exoplanets table links to `/exoplanets/<encoded pl_name>`.
- Edge cases: URL encoding + missing record UI.
- Tests: add server-side lookup tests for planet detail.
- Note: REST single-record endpoint is not required now; add as future idea instead.

### Recent completions
- Measurement groups/err/lim rendering in tables + metadata tooltips.
- Unified server table pipeline via `get_table_data`.
- First-column text filter (server-side) with `filter` param.

### Next (after planet detail)
- Finalize `lim` indication styling for measurement cells.
- Per-column filters in tables (server + frontend).

## Status (Jan 29, 2026)

- VOTable parsing/loader/conversion moved into `exo-cli`.
- `exo-core` restored table modules + metadata helpers; web + CLI separated.

## Next Task: Per-Column Filters in Tables

**Scope (high-level)**
- Frontend: render a filter row aligned to columns; persist filters in URL/state.
- Backend: accept filters in handlers; apply filters before totals/pagination.

**Decisions needed**
- Filter format in URL.
- Filter semantics for strings/numbers.

**Acceptance criteria**
- Each visible column has a filter input directly beneath the header.
- Filters update rows + totals and persist across sort/paging.
- Clearing filters restores the unfiltered dataset.

**Notes**
- Detailed refactor notes live in `specs/review.md`.
