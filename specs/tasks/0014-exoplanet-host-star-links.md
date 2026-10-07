---
id: 0014-exoplanet-host-star-links
status: draft
scope: [src/table/table.rs, src/components/catalog_table.rs, src/components/exoplanets_table/page.rs]
created: 2026-10-07
---
# Task: exoplanet host star links

## Plan

- [ ] Extend shared table link configuration to support planet and host columns with separate destination routes.
- [ ] Render exoplanet `hostname` cells as links to `/stellarhosts/{hostname}`, preserving the active locale and existing planet links.
- [ ] Add focused link-generation coverage, update the frontend specification, and verify the split web build.

## Acceptance

- [ ] `cargo test --locked -p exodata-web --lib table` passes with coverage for both link destinations, locale prefixes, encoded names, and missing host values.
- [ ] `cargo fmt --all` and `cargo leptos build --split` pass.
- [ ] `git diff --check` and `specdev check` pass without new task-specific warnings.

## Manual checks

- [ ] Open `/exoplanets`: a Host star value navigates to its stellar-host detail page, and Planet name still navigates to its planet detail page.
- [ ] Repeat on `/zh-CN/exoplanets` and `/ja/exoplanets`; both destination links preserve the locale after direct load and client navigation.
- [ ] Confirm host links remain correct after sorting, filtering, pagination, and custom column selection; the stellarhosts table retains its existing host links.

## Context

- User requested this task on 2026-10-07 after noticing Host star values were plain text in the exoplanet table.
- Current shared `Table` configuration accepts a single `link_column` and `link_base`; the exoplanet page configures `pl_name` only. Supporting `hostname` requires distinct destination routes for the two columns.
- Use the original host name for display and a correctly encoded path segment for navigation. Null or empty host values must not create a detail link.
- Scope is limited to catalog link rendering and its configuration; data, column labels, filtering, and sorting remain unchanged. Record the resulting contract in `specs/web-frontend.md` during implementation.

## Log

- 2026-10-07 created: exoplanet host star links
- 2026-10-07 scope add src/table/table.rs — Support host and planet detail links in the exoplanet table
- 2026-10-07 scope add src/components/catalog_table.rs — Support host and planet detail links in the exoplanet table
- 2026-10-07 scope add src/components/exoplanets_table/page.rs — Support host and planet detail links in the exoplanet table
