# Current Context

## Current task

### Recently completed
- Exoplanet detail page with duplicate-row records by `pl_name`.
- Table links from exoplanets → `/exoplanets/:pl_name`.
- Server helper + test for multi-row planet detail.
- Navbar active state now includes detail routes.
- Stellar host detail lists all matching records (multi-row), with summary + record list.

### Next
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
