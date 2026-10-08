---
id: 0014-exoplanet-host-star-links
status: approval
scope: [src/table/table.rs, src/components/catalog_table.rs, src/components/exoplanets_table/page.rs]
created: 2026-10-07
---
# Task: exoplanet host star links

## Plan

- [x] Extend shared table link configuration to support planet and host columns with separate destination routes.
- [x] Render exoplanet `hostname` cells as links to `/stellarhosts/{hostname}`, preserving the active locale and existing planet links.
- [x] Add focused link-generation coverage, update the frontend specification, and verify the split web build.

## Acceptance

- [x] `cargo test --locked -p exodata-web --lib table` passes with coverage for both link destinations, locale prefixes, encoded names, and missing host values.
- [x] `cargo fmt --all` and `cargo leptos build --split` pass.
- [x] `git diff --check` and `specdev check` pass without new task-specific warnings.

## Manual checks

- [ ] Open `/exoplanets`: a Host star value navigates to its stellar-host detail page, and Planet name still navigates to its planet detail page.
- [ ] Repeat on `/zh-CN/exoplanets` and `/ja/exoplanets`; both destination links preserve the locale after direct load and client navigation.
- [ ] Confirm host links remain correct after sorting, filtering, pagination, and custom column selection; the stellarhosts table retains its existing host links.

## Context

- User authorized implementation as a separate commit-sized unit on 2026-10-08 after committing the prior work. Use a column-to-route map in Table and a small optional additional-links configuration in CatalogTableResult; the exoplanet caller adds hostname → /stellarhosts/. Keep original values for display and use the shared encoder/locale helper for navigation.
- User requested this task on 2026-10-07 after noticing Host star values were plain text in the exoplanet table.
- Current shared `Table` configuration accepts a single `link_column` and `link_base`; the exoplanet page configures `pl_name` only. Supporting `hostname` requires distinct destination routes for the two columns.
- Use the original host name for display and a correctly encoded path segment for navigation. Null or empty host values must not create a detail link.
- Scope is limited to catalog link rendering and its configuration; data, column labels, filtering, and sorting remain unchanged. Record the resulting contract in `specs/web-frontend.md` during implementation.

## Findings

- Verified on 2026-10-08: all 45 focused table tests passed, including both destinations in all three locales, reserved-character encoding, original-name retention, and null/empty/whitespace-only values. Formatting, split SSR/hydration/Tailwind build, workspace/all-targets/all-features Clippy with warnings denied, and diff checks passed. Specdev reports zero errors and 2 existing global warnings; proc-macro-error2 future-compatibility warning remains informational.
- Live SSR checks verified both link destinations and encoded original names on English/Chinese/Japanese default tables and custom hostname/pl_name selections sorted by hostname. Existing stellarhosts links remain correct. Browser click/client-navigation checks remain human-owned.

## Review

- Reviewed the three-file implementation against the task. Table now uses column routes; CatalogTableResult retains its primary-link caller contract and accepts optional additional static routes. Only the exoplanet caller adds the host route. Successful links keep the existing Leptos A styling and original text; empty values use the ordinary cell path. No data, heading, sorting, filtering, pagination, dependency, or other page changes.

## Summary

Link Host star values in the exoplanet table to locale-aware stellarhost profiles while preserving planet links and plain missing-value cells.

## Log

- 2026-10-07 created: exoplanet host star links
- 2026-10-07 scope add src/table/table.rs — Support host and planet detail links in the exoplanet table
- 2026-10-07 scope add src/components/catalog_table.rs — Support host and planet detail links in the exoplanet table
- 2026-10-07 scope add src/components/exoplanets_table/page.rs — Support host and planet detail links in the exoplanet table
- 2026-10-08 advance draft → ready
- 2026-10-08 scope approved: src/table/table.rs, src/components/catalog_table.rs, src/components/exoplanets_table/page.rs
- 2026-10-08 advance ready → in-progress/implement
- 2026-10-08 advance in-progress/implement → in-progress/verify
- 2026-10-08 advance in-progress/verify → in-progress/review
- 2026-10-08 advance in-progress/review → approval
