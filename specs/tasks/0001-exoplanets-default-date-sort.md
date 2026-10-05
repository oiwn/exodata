---
id: 0001-exoplanets-default-date-sort
status: approval
scope: [src/components/exoplanets_table/page.rs, end2end/tests/smoke.spec.ts, specs/web-frontend.md, specs/web-backend.md, src/table/*.rs, src/server/**/*.rs]
created: 2026-10-05
source: https://github.com/oiwn/exodata/issues/143
attempts: 1
---
# Task: exoplanets default date sort

## Plan

- [x] Declare file scope through specdev and confirm the existing browser fixture contains `rowupdate` and `releasedate` as strings.
- [x] Document the approved website default and its interaction with URL state in the frontend spec; document any changed shared sorting behavior in the backend spec before changing it.
- [x] Add focused regression tests with synthetic dated records for default ordering, missing dates, explicit overrides, and pagination totals; add query-state coverage for clearing and column changes.
- [x] Implement `rowupdate` descending whenever the exoplanets website has no explicit sort, retain undated records last, and preserve source order for equal dates. Add the Updated default column and show the effective sort across SSR and hydration, including when custom columns hide the date.
- [x] Add a focused browser smoke test for direct loading, client navigation, sort overrides, and URL reloads; hand over the manual checks after the split build, before expanding automated verification.

## Acceptance

- [x] `cargo test --locked -p exodata-web --lib table` passes, including regressions for `rowupdate` descending despite older release dates, null dates retained last with correct totals, stable equal-date pagination, explicit sort overrides, clearing to the default, and default sorting with filters, pagination, or custom columns.
- [x] `cargo test --locked -p exodata-web --lib server::tests` passes; shared backend changes preserve existing REST sorting contracts unless explicitly approved otherwise.
- [x] `cargo leptos build --split` passes for SSR and hydration; complete the manual checks below before expanding automated verification.
- [x] With the fixture-backed app running, from `end2end/`, `npx playwright test --grep "exoplanets date sort|catalog table interactions preserve query state" --reporter=html` passes. Name the new focused test to match this filter and ensure its input contains the date cases it asserts.
- [x] `specdev check` and the scoped `git diff --check` pass without new errors or task-specific quality warnings.

## Context

- Goal from [issue #143](https://github.com/oiwn/exodata/issues/143): open the exoplanets website table with the newest parameter records first when no explicit sort is selected, preserve user sort URLs, and show the active date sort.
- The user authorized implementation. The website query path and query-state transitions implement the agreed default; all automated acceptance checks passed, and the user confirmed the browser flow and green report.
- Scope globs group related files for specdev. Intended table edits are limited to `src/table/query_navigation.rs` and `src/table/table.rs`; intended server edits are limited to `src/server/functions/tables.rs`, `src/server/data/tables.rs`, `src/server/cache.rs`, and `src/server/tests.rs`. Other files matched by those globs are outside the planned work.
- The table contains reference-based parameter records; this task does not introduce one-row-per-planet deduplication.
- Dataset counts are inspection observations, not acceptance criteria. Local VOTable data and Parquet footer metadata were inspected during this review; no data refresh was run.

## Findings

- The source prerequisite is already satisfied: `Justfile` downloads `ps`, and [data-management.md](../data-management.md) specifies `ps` and both date fields. The issue's `pscomppars` warning is stale; no download-recipe change is currently needed.
- Commit `720d684` (2026-09-07, specs for descriptions generation) changed the download source from `pscomppars` to `ps` and documented the date fields.
- Inspection on 2026-10-05 found 40,144 rows and 355 fields in both local exoplanet files. The VOTable contains 6,360 distinct planets and one blank `rowupdate`; Parquet stores the date columns as UTF-8 strings with one null `rowupdate` and no null `releasedate`. VOTable `rowupdate` values use `YYYY-MM-DD`; `releasedate` has both date-only and timestamp values.
- [NASA column definitions](https://exoplanetarchive.ipac.caltech.edu/docs/API_PS_columns.html) distinguish `rowupdate` (last planet-parameter update) from `releasedate` (parameter-set public release); both are PS fields. Verified 2026-10-05.
- Before this task, `src/components/exoplanets_table/page.rs` used seven default display columns, neither date included. Shared initialization in `src/table/query_navigation.rs` left sorting unset and defaulted the order to ascending.
- `src/server/data/tables.rs` projects selected columns before sorting, ignores a sort key outside that selection, and filters out null sort values. Merely setting a default date sort would therefore either do nothing or remove undated rows after adding the date column.
- The shared query state cycles ascending, descending, then cleared; removing the sorted display column clears the sort. Pagination and filtering preserve the remaining sort state. These behaviors are documented in [web-frontend.md](../web-frontend.md).
- Shared backend sorting also serves REST and cached queries. Keep the website default at the website boundary; do not silently change API defaults or all datasets' null handling.
- The existing browser fixture manifest contains `rowupdate` and `releasedate` as strings. Synthetic regression inputs must cover null dates and ties independently of the fixture's sampled values or live dataset completeness.

## Verification findings

- `cargo test --locked -p exodata-web --lib table`: 37 passed, including the new date-ordering, stable pagination, explicit override, query-transition, and website/REST cache-isolation tests.
- `npm run typecheck` from `end2end/`: passed. The focused browser smoke results are recorded below.
- `cargo leptos build --split`: passed for SSR and WASM hydration. `specdev check`: zero errors and no task-specific warnings. Scoped `git diff --check`: passed.
- The user reported that the exoplanets flow works. `cargo test --locked -p exodata-web --lib server::tests`: 25 passed.
- First Playwright run found a test-selector error: counting all `thead th` elements included the filter row. The report snapshot confirmed one visible data column; the assertion now counts only the first header row. The other test timed out while the local app was unavailable. Both failures were resolved in the rerun below.
- Playwright rerun against the fixture-backed local server: both focused tests passed (2 passed, 32.6 seconds), covering SSR, hydration, homepage navigation, overrides, history, hidden date columns, reloads, filters, and table interactions. The HTML report at `http://127.0.0.1:9323` now contains the passing run. Human-owned Manual checks remain unticked.
- Manual-check ownership stays with the user. After the split build, hand over the runnable command and URLs before running further automated checks, following the repository checks workflow.
- On 2026-10-06 the user confirmed all checks are green. Final diff review and `git diff --check` passed; `specdev check` reports zero errors and no task-specific warnings. Manual checkboxes remain user-owned.

## Review

- Approved: the diff matches the agreed website behavior and declared implementation files. Update-date ordering retains null dates and stable ties before projection; the separate website cache namespace preserves REST behavior.
- Query transitions, explicit overrides, hidden Updated columns, history, SSR, hydration, and reloads are covered by the passing focused checks. No implementation changes were needed during final review.
- Scope remains exoplanets only. The stellarhosts dataset has no update field, and the user accepted leaving its behavior unchanged. Commit, merge, and completion archival remain user-managed.

## Summary

Default the exoplanets website to newest update dates first, with an Updated column, undated records last, stable ties, and preserved sort and column URL state.

## Decisions

- **Primary date:** the user selected `rowupdate` descending so recently updated parameter records appear first, including updates to older released records. Do not substitute `releasedate` for a missing update date.

- **Missing dates:** retain records without `rowupdate` after dated records in the default view, including them in pagination totals, with no `releasedate` fallback. One local record is missing its update date; that count is snapshot evidence, not an acceptance criterion.
- **Dataset contract:** `rowupdate` is present in the current dataset and browser fixture. No new missing-column fallback or special recovery behavior is required by this task.
- **Visible state:** include `rowupdate` in the default display columns with the label Updated and an active descending indicator. When custom columns hide it, retain a visible indication of the effective date sort without forcing it back into the display selection. General heading localization remains tracked separately in task 0008.
- **Clear sorting:** treat a cleared sort as a return to `rowupdate` descending on the exoplanets website. Preserve explicit user sorting and do not change the shared stellarhosts cycle implicitly.
- **Custom columns:** the user's clarification selected the date default whenever `sort` is absent, including URLs with `page`, `filter`, or `columns`. Fetch the date key internally when it is hidden; sort before final projection or use a narrow default-sort path. Explicit sort parameters override the default and remain preserved in navigation URLs.
- **Date ties:** preserve original source-row order within equal update dates so repeated page requests remain consistent. Test ties spanning page boundaries; no extra visible sort column is required.

## Manual checks

- [ ] Run `EXO_DATA_DIR=end2end/runtime-data cargo leptos watch --split`; open `http://127.0.0.1:3000/exoplanets` directly and through homepage navigation. Confirm the approved descending date order and visible active sort after hydration.
- [ ] Open `/exoplanets?sort=disc_year&order=asc`, filter and paginate, then reload and use browser back/forward. Confirm the explicit sort and URL state remain consistent.
- [ ] Clear an explicit sort and confirm `rowupdate` descending is restored. Open URLs with only `page`, `filter`, or custom `columns` excluding `rowupdate`; confirm the date default and visible active-sort indication survive reload. Confirm `/stellarhosts` retains its current behavior.
- [ ] Check missing-date records remain reachable with correct totals using controlled data; verify no duplicate or skipped records appear across pages containing equal dates.

## Log

- 2026-10-05 created: exoplanets default date sort
- 2026-10-05 set source https://github.com/oiwn/exodata/issues/143
- 2026-10-05 scope add src/components/exoplanets_table/page.rs — Initialize and restore the website date default; show active sorting when Updated is hidden
- 2026-10-05 scope add src/table/{query_navigation.rs,table.rs} — Support exoplanet query transitions and the Updated heading without changing stellarhosts behavior
- 2026-10-05 scope add src/server/{functions/tables.rs,data/tables.rs,cache.rs,tests.rs} — Keep website ordering, projection, null retention, and cache semantics distinct from REST; add regressions
- 2026-10-05 scope add end2end/tests/smoke.spec.ts — Verify date defaults, explicit overrides, hidden columns, SSR and navigation
- 2026-10-05 scope add specs/web-frontend.md — Document the approved website date default and query behavior
- 2026-10-05 scope add specs/web-backend.md — Document website-specific ordering and cache behavior
- 2026-10-05 advance draft → ready
- 2026-10-05 scope approved: src/components/exoplanets_table/page.rs, src/table/{query_navigation.rs,table.rs}, src/server/{functions/tables.rs,data/tables.rs,cache.rs,tests.rs}, end2end/tests/smoke.spec.ts, specs/web-frontend.md, specs/web-backend.md
- 2026-10-05 scope rm src/table/{query_navigation.rs,table.rs}
- 2026-10-05 scope add src/table/*.rs — Query transitions and Updated heading; intended edits limited to query_navigation.rs and table.rs
- 2026-10-05 scope rm src/server/{functions/tables.rs,data/tables.rs,cache.rs,tests.rs}
- 2026-10-05 scope add src/server/**/*.rs — Website table ordering and cache isolation; intended edits limited to functions/tables.rs, data/tables.rs, cache.rs, and tests.rs
- 2026-10-05 advance ready → in-progress/implement
- 2026-10-05 advance in-progress/implement → in-progress/verify
- 2026-10-05 advance in-progress/verify → in-progress/fix (attempts 1)
- 2026-10-05 advance in-progress/fix → in-progress/verify
- 2026-10-06 advance in-progress/verify → in-progress/review
- 2026-10-06 advance in-progress/review → approval
