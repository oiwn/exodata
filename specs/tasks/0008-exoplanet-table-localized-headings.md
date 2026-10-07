---
id: 0008-exoplanet-table-localized-headings
status: blocked
scope: [style/components/*.css, src/components/**/*.rs, src/server/*.rs, locales/*.json, docs/mcp.md, src/table/*.rs]
created: 2026-10-05
source: https://github.com/oiwn/exodata/issues/153
attempts: 3
blocked_reason: Implementation verified and latest table design visually approved by the user; awaiting shared batch PR merge
---
# Task: exoplanet table localized headings

## Plan

- [x] Record the approved eight-heading boundary and declare scoped rendering/locale changes without altering metadata or NASA identifiers.
- [x] Add English, Chinese, and Japanese display labels for `pl_name`, `hostname`, `discoverymethod`, `disc_year`, `pl_orbper`, `pl_rade`, `pl_bmasse`, and `rowupdate`; opt the exoplanet table into those labels while retaining existing fallback headings and stellarhost behavior.
- [x] Cover localized default headings and custom-key fallback with focused tests; verify table checks, formatting, locale JSON, and split SSR/hydration build before handing over manual review.
- [x] Address user screenshot feedback: prevent header/date wrapping with horizontal table scrolling; translate catalog controls/selector/page text and preserve the selected locale during table navigation. Keep the eight-heading boundary for scientific columns.
- [x] Tighten horizontal cell/header spacing, add subtle header/filter-cell borders, supply explanatory default-heading tooltips with metadata units, and give the gradient catalog title enough descender room. Keep scrolling and header font size unchanged.
- [x] Use English “Disc. year” and display Transit Timing Variations as TTV; replace native delayed table tooltips with immediate hover/focus descriptions while retaining full method names and raw data.

## Acceptance

- [x] `cargo test --locked -p exodata-web --lib table` passes, including eight-heading locale coverage and custom-column fallback.
- [x] `cargo fmt --all` and `cargo leptos build --split` pass; all three locale JSON files parse with the same eight new keys.
- [x] `git diff --check` and `specdev check` pass without new task-specific warnings. Sorting/query column keys, metadata tooltips/units, selected-column identifiers, and catalog values are unchanged.
- [x] Follow-up table/locale checks and split build pass; shared control strings are present in all three locale resources, navigation retains locale prefixes and raw field/query keys, and headers remain single-line with original scientific formula case.
- [x] Focused tooltip/table tests and split build pass; all eight default headings have localized explanatory tooltips, metadata units are preserved, headers remain single-line, and the density/title changes stay within the requested styling scope.
- [x] Focused abbreviation/table tests and split build pass; TTV formatting is limited to the discovery-method display cell, full names remain available, and tooltip CSS has no show delay.

## Manual checks

- [ ] Hover/focus headings and a TTV cell: descriptions/full method name should appear immediately. Confirm English Disc. year, unmodified Chinese/Japanese year headings, and unchanged raw sorting/filter keys.
- [ ] Refresh the English catalog at desktop width: inspect denser columns, header borders, and the full descender in Catalog. Hover each default heading for its description/units. Check Chinese/Japanese and mobile horizontal scrolling; header font size is unchanged.
- [ ] On `/zh-CN/exoplanets` and `/ja/exoplanets`, confirm headings/dates stay on one line (scroll horizontally when needed), table/page/selector/filter/pagination controls are translated, and Next/sort/column/filter interactions preserve the selected locale. Inspect desktop and mobile, plus shared controls on `/stellarhosts`.
- [ ] Run `cargo leptos watch --split`; open `/exoplanets`, `/zh-CN/exoplanets`, and `/ja/exoplanets` at `http://127.0.0.1:3000`. Confirm readable default headings in the selected language on direct load and after hydration.
- [ ] Select a custom column such as `pl_eqt`; confirm its existing scientific heading remains unchanged, tooltips/units still work, and sorting uses NASA field keys. Check `/stellarhosts` retains its existing headings.

## Summary

Localize the eight default exoplanet display headings while preserving scientific column identifiers and custom headings. Include translated catalog controls, locale-preserving navigation, and single-line table text.

## Context

- User follow-up on 2026-10-07: shorten only the English Discovery year label to “Disc. year”, abbreviate the wide Transit Timing Variations display value to TTV, and eliminate the browser-native initial tooltip delay. Data values, exports, filters, and sort keys retain the full scientific source strings.
- Latest screenshot feedback on 2026-10-07 approves scrolling/header size and requests less horizontal padding, description tooltips for at least default columns, subtle header borders, and room for the title's clipped `g`. `style/components/catalog-table.css` is now an intended edit under the existing CSS scope. Default tooltip text is UI copy; underlying scientific metadata remains unchanged.
- User-expanded interface follow-up on 2026-10-07: the screenshot shows per-character CJK header wrapping and English Next/Select Columns/page controls. Additional edits are authorized in `src/components/column_selector.rs`, both catalog page components, and the shared loading overlay; all are covered by existing grouped scope. This expands UI chrome translation, not the eight-heading scientific-data boundary. Locale-aware navigation is required so translated controls do not return users to English after interaction.
- [Issue #153](https://github.com/oiwn/exodata/issues/153) requests proper table headings and language translations. On 2026-10-07 the user narrowed implementation to the eight default exoplanet headings, preferring scientific names for other columns. Do not expand this into translating all selectable columns or metadata descriptions.
- New edits are limited to `src/table/table.rs`, `src/components/catalog_table.rs`, and `locales/en.json`, `locales/zh-CN.json`, `locales/ja.json`. Grouped scopes retain earlier uncommitted batch changes; other matched files are not new edit scope.
- Labels are presentation-only and do not replace NASA keys, unit strings, row values, query parameters, or metadata payloads. The mass heading explicitly retains the mass-or-M-sin-i distinction. The user's subsequent screenshot feedback authorized shared selector/table chrome translation; scientific-column localization remains limited to the default eight.

## Findings

- Short-label/immediate-tooltip follow-up verified on 2026-10-07: 44 focused table tests passed, including display-only TTV formatting and original-value retention. Formatting, split SSR/hydration/Tailwind build, diff checks, and immediate tooltip CSS checks passed. The English heading is Disc. year; other languages' year headings are unchanged. Header descriptions and TTV full-name hints show on hover or keyboard focus without native title attributes/timers. Header Enter/Space sorting supports the new focusable headers. Visual hover/focus checks remain with the user.
- Density/tooltip/title follow-up verified on 2026-10-07: 43 focused table tests passed, including all default tooltip descriptions without source metadata and metadata-unit retention. Formatting, matching eight-description locale dictionaries, split SSR/hydration/Tailwind build, and diff checks passed. Cell horizontal padding is 24px → 12px per side, header/measurement gaps 8px → 4px, and the filter input is limited to 8rem. Header borders and catalog-title line height/bottom padding were added; scrolling and header font size remain unchanged. Visual inspection remains with the user.
- Screenshot follow-up verified on 2026-10-07: 41 focused table tests passed, including new exoplanet/stellarhost locale-navigation cases with raw scientific sort/column keys. Formatting, split SSR/hydration/Tailwind build, matching 31-key control dictionaries/interpolation arguments, and `git diff --check` passed. Numeric summaries use the existing library's view interpolation; no localization configuration/dependency change was needed. Specdev reports zero errors and 25 existing warnings. The corrected layout and interactive controls await the user's browser check.
- Verification on 2026-10-07: 39 focused table tests passed, including the two new heading tests and existing sort/query behavior. Formatting, split SSR/hydration/Tailwind build, exact eight-key locale JSON checks, and `git diff --check` passed. Specdev reports zero errors and 25 existing warnings, none for task 0008. Browser checks remain with the user; no new server or browser automation was started.
- The table currently formats a few hard-coded stellarhost labels; most exoplanet keys render verbatim. Metadata descriptions/units are used for tooltips, not headings. The current local export has 189 selectable base columns but only 91 descriptions; these are inspection observations, not fixture or completeness requirements.
- Default exoplanet heading translations are enabled only for the exoplanet catalog surface. Other columns use the existing formatter, and stellarhost headings remain unchanged.

## Review

- User visually approved the latest table layout, abbreviated labels, and immediate tooltips on 2026-10-07 (“looks good”) and requested session bookkeeping. Implementation and visual review are complete; park pending the shared batch PR merge. Manual checklist boxes remain human-owned and are not treated as proof of checks the user did not individually report.
- Latest follow-up reviewed: only the English year heading is shortened, and only Transit Timing Variations discovery-method display cells use TTV. Full method text stays in a hover/focus tooltip and in the unmodified data value. Table descriptions use immediate CSS tooltips with ARIA associations and edge-side positioning; underlying fields, filtering, sorting, and exports are unchanged. Build/tests pass and the task is ready for the requested visual check.
- Latest requested styling reviewed: the shared table is more compact horizontally, header/filter cells have subtle borders, and native tooltips explain all eight default columns in each locale with raw field names and available unit strings. Caller-provided descriptions retain precedence and scientific metadata payloads are untouched. Catalog-title descenders get extra line-height/bottom room. No scroll redesign or header font-size change was introduced.
- Follow-up reviewed against the screenshot and expanded interface request: headers/date cells no longer wrap into vertical text, scientific formula case is preserved, and wide tables use the existing horizontal scroll container. Page, selector, filter, pagination, loading/error-prefix labels are translated for all three locales. Both catalogs' interactions, numeric page links, overview links, and object links retain locale prefixes. Original custom-column names, metadata descriptions/units, and values remain unchanged; raw server errors and page metadata remain outside this pass.
- Reviewed the five-file implementation against the narrowed scope: eight display labels per locale, an optional exoplanet-only heading flag, and formatter tests. NASA metadata and field names, tooltips, units, values, sort callbacks/query construction, selector content, and stellarhost aliases retain their existing behavior. The mass label preserves M sin i. Ready for local manual review after the successful split build.

## Log

- 2026-10-05 created: exoplanet table localized headings
- 2026-10-05 set source https://github.com/oiwn/exodata/issues/153
- 2026-10-07 scope add style/components/*.css — Eight localized exoplanet headings plus retained authorized batch changes; exact edit limits recorded in Context
- 2026-10-07 scope add src/components/**/*.rs — Eight localized exoplanet headings plus retained authorized batch changes; exact edit limits recorded in Context
- 2026-10-07 scope add src/server/*.rs — Eight localized exoplanet headings plus retained authorized batch changes; exact edit limits recorded in Context
- 2026-10-07 scope add locales/*.json — Eight localized exoplanet headings plus retained authorized batch changes; exact edit limits recorded in Context
- 2026-10-07 scope add docs/mcp.md — Eight localized exoplanet headings plus retained authorized batch changes; exact edit limits recorded in Context
- 2026-10-07 scope add src/table/*.rs — Eight localized exoplanet headings plus retained authorized batch changes; exact edit limits recorded in Context
- 2026-10-07 advance draft → ready
- 2026-10-07 scope approved: style/components/*.css, src/components/**/*.rs, src/server/*.rs, locales/*.json, docs/mcp.md, src/table/*.rs
- 2026-10-07 advance ready → in-progress/implement
- 2026-10-07 advance in-progress/implement → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 advance approval → in-progress/fix (attempts 1)
- 2026-10-07 advance in-progress/fix → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 advance approval → in-progress/fix (attempts 2)
- 2026-10-07 advance in-progress/fix → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 advance approval → in-progress/fix (attempts 3)
- 2026-10-07 advance in-progress/fix → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 blocked: Implementation verified and latest table design visually approved by the user; awaiting shared batch PR merge
