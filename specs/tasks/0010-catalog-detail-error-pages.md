---
id: 0010-catalog-detail-error-pages
status: approval
scope: [src/components/**/*.rs, src/server/**/*.rs, src/error_template.rs, src/table/table.rs, locales/*.json]
created: 2026-10-05
source: https://github.com/oiwn/exodata/issues/155
attempts: 1
---
# Task: catalog detail error pages

## Plan

- [x] Distinguish missing objects from internal lookup failures; return optional detail payloads from the two detail server functions without parsing error messages.
- [x] Render the shared branded error template for absent detail records (404) and actual loading failures (500), with no raw server error in the page; retain successful detail rendering.
- [x] Verify missing/malformed lookup cases and error rendering/status, formatting, and the split build; hand over direct-load/client-navigation manual checks.

## Acceptance

- [x] Focused `cargo test --locked -p exodata-web --lib detail` and `cargo test --locked -p exodata-web --lib error_template` pass for missing objects, operational failures, and correct branded 404/500 status.
- [x] `cargo fmt --all` and `cargo leptos build --split` pass.
- [x] `git diff --check` and `specdev check` pass without task-specific warnings.

## Manual checks

- [ ] With `cargo leptos watch --split`, open `/stellarhosts/fdfdsfsdfs` and `/exoplanets/fdfdsfsdfs`: see the branded 404 without raw server errors; verify HTTP 404. Open valid host/planet profiles and navigate from catalog links; confirm successful profiles and return links work.
- [ ] Check missing profiles after client navigation and under `/zh-CN` and `/ja`; recovery links should retain the locale. Internal failures show a branded 500 rather than a false 404.

## Context

- Screenshot follow-up on 2026-10-07: the error page was nested in the detail shell, causing duplicate back links, stacked backgrounds, and excessive inset. Render errors directly at the route root. Preserve the detail shell and back link around successful content and the loading shell around pending content.
- Issue #155 reports a nonexistent stellar host rendering “Error Loading Host” plus the raw server-function error. User authorized task 0010 after visually approving task 0009 on 2026-10-07.
- Use typed lookup errors internally; absence becomes `Ok(None)` for detail server functions while operational failures remain errors. The website renders the existing branded ErrorTemplate and sets SSR status to 404/500. Detail export payloads and their existing string error contracts stay unchanged.
- Exact implementation scope: `src/server/data/details.rs`, `src/server/functions/details.rs`, `src/components/exoplanet_detail/page.rs`, `src/components/stellarhost_detail/page.rs`, and `src/error_template.rs`, with focused tests in those modules. Grouped scope also retains previously approved batch files; do not change unrelated components/server modules, tables, or locale copy.
- No new localization pass: reuse existing English error copy and preserve locale in recovery links. Valid profiles and successful description/planet lookup behavior remain unchanged.

## Summary

Render branded 404 pages for missing stellar hosts and exoplanets and branded 500 pages for internal loading failures, with correct SSR status and no raw server error text.

## Findings

- Standalone-layout follow-up verified on 2026-10-07: formatting, split SSR/hydration/Tailwind build, and diff checks passed. Both missing-object URLs return 404 with only Back to Overview and no profile-shell class/back link. The valid Kepler-22 host remains 200 with its profile shell and Back to Stellar Hosts. Specdev reports zero errors and 6 existing warnings. Visual review remains with the user; prior lookup/error-status tests were not rerun for this layout-only correction.
- Live SSR checks on 2026-10-07: `/stellarhosts/fdfdsfsdfs` and `/exoplanets/fdfdsfsdfs` render Not Found with HTTP 404 and no old raw error block. `/stellarhosts/Kepler-22` and `/exoplanets/Kepler-22%20b` return HTTP 200. `/ja/stellarhosts/fdfdsfsdfs` returns 404 with noindex and locale-preserving recovery links. Browser visual/hydration/client-navigation checks remain human-owned.
- Formatting and split SSR/hydration/Tailwind build passed. All 30 focused detail checks and 2 standalone error-template checks passed, including operational-schema failures, absent objects, actual Axum 404/500 responses, status-specific copy, and Japanese recovery links. Test harness uses normal SSR route initialization and a full HTML document. Diff checks passed; specdev reports zero errors and 6 existing warnings. The existing proc-macro-error2 future-compatibility warning remains.

## Review

- Screenshot review found the shared error template rendered inside the successful-profile shell. The two page components now select the complete profile shell only for successful payloads; missing/internal errors render the standalone shared template. Split build and live wrapper/navigation checks passed. Loading retains the profile background/container and spinner; profile recovery navigation appears with successful content.
- Reviewed the five implementation files against the plan. Typed lookup errors preserve string messages for existing export callers; only the two UI detail functions change to optional payloads. Missing host results take precedence over related-planet loading so an absent host remains 404. Query/conversion and related-planet failures render 500; missing generated descriptions retain their existing optional behavior.
- Shared error rendering retains the branded layout and adds status-specific copy, error titles/noindex, and locale-preserving recovery links. Successful profile content is unchanged. Live valid/missing SSR checks pass; browser visual/hydration/client-navigation checks remain with the user. No staging, commits, or archival performed.

## Log

- 2026-10-05 created: catalog detail error pages
- 2026-10-05 set source https://github.com/oiwn/exodata/issues/155
- 2026-10-07 scope add src/components/**/*.rs — Detail lookup classification, branded 404/500 rendering, focused regression checks; retain approved uncommitted batch
- 2026-10-07 scope add src/server/**/*.rs — Detail lookup classification, branded 404/500 rendering, focused regression checks; retain approved uncommitted batch
- 2026-10-07 scope add src/error_template.rs — Detail lookup classification, branded 404/500 rendering, focused regression checks; retain approved uncommitted batch
- 2026-10-07 scope add src/table/table.rs — Detail lookup classification, branded 404/500 rendering, focused regression checks; retain approved uncommitted batch
- 2026-10-07 scope add locales/*.json — Detail lookup classification, branded 404/500 rendering, focused regression checks; retain approved uncommitted batch
- 2026-10-07 advance draft → ready
- 2026-10-07 scope approved: src/components/**/*.rs, src/server/**/*.rs, src/error_template.rs, src/table/table.rs, locales/*.json
- 2026-10-07 advance ready → in-progress/implement
- 2026-10-07 advance in-progress/implement → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 advance approval → in-progress/fix (attempts 1)
- 2026-10-08 advance in-progress/fix → in-progress/verify
- 2026-10-08 advance in-progress/verify → in-progress/review
- 2026-10-08 advance in-progress/review → approval
