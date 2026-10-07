---
id: 0013-catalog-default-value-abbreviations
status: blocked
scope: [src/table/table.rs, locales/en.json, locales/zh-CN.json, locales/ja.json]
created: 2026-10-07
attempts: 2
blocked_reason: User visually approved headings and filter layout; awaiting shared batch PR merge
---
# Task: catalog default value abbreviations

## Plan

- [x] Audit default catalog fields and document display-only abbreviation rules.
- [x] Shorten the English discovery-method heading and wide method values; reuse immediate hover/focus tooltips for every abbreviated value.
- [x] Verify focused table tests, formatting, and the split web build; review the scoped diff.

## Acceptance

- [x] `cargo test --locked -p exodata-web --lib table` passes, covering abbreviation/full-value retention, fallback values, compact method/mass headings, and full mass tooltip text.
- [x] `cargo fmt --all` and `cargo leptos build --split` pass.
- [x] `git diff --check` and `specdev check` pass without task-specific warnings.

## Manual checks

- [ ] Confirm the filter input starts at the same left inset as the first heading and row values, with normal-weight text and a compact height, on both catalog tables. Typing and Enter/blur still apply the filter.
- [ ] On `/exoplanets`, hover and keyboard-focus an RV/TTV cell: the original full method appears immediately. Confirm Disc. method, Mass with its full explanatory tooltip, and unchanged filter/sort behavior. Check `/zh-CN/exoplanets` and `/ja/exoplanets` retain their localized headings, including compact mass labels.

## Context

- User screenshot feedback on 2026-10-07 confirms the compact headings and requests correcting filter disposition. The first filter-row th inherited browser centering/bold styling; explicitly left-align it, use normal weight, render the input as a block, and reduce input vertical padding while retaining its width and event handlers.
- User addition to the current batch on 2026-10-07: inspect default field widths, abbreviate wide values with full-text tooltips, and use English “Disc. method”; Radial Velocity is the motivating value.
- The default exoplanet columns contain object names, discovery method/year, measurements, and an update date. Stellarhost defaults contain an object name and measurements. Preserve complete names, dates, numeric precision, and mass provenance; shorten method category text and the localized mass headings in this follow-up.
- User correction on 2026-10-07: display “Mass” rather than “Mass or M sin i”; keep the full distinction in the tooltip. Apply equivalent compact Chinese/Japanese headings and retain all existing tooltip text and source identifiers.
- Display mapping: Radial Velocity → RV; Transit Timing Variations → TTV; Eclipse Timing Variations → ETV; Pulsation Timing Variations → PTV; Orbital Brightness Modulation → OBM; Disk Kinematics → DK. Other method strings retain their full text. All mapped values expose their original string through the existing immediate hover/focus tooltip. Raw values, filters, exports, and sorting remain unchanged.
- This addition is implemented ahead of queued task 0009; tasks 0001–0008 remain parked pending the user-managed shared batch merge.

## Findings

- Batch lint correction on 2026-10-08: collapse the nested discovery-method/abbreviation condition into a Rust let-chain. Display behavior is unchanged. `cargo fmt --all`, `cargo lx --locked -- -D warnings` (workspace/all-targets/all-features), and `git diff --check` passed. The existing proc-macro-error2 future-compatibility warning remains informational.
- Filter-layout correction verified on 2026-10-07: all 44 table tests, formatting, split SSR/hydration/Tailwind build, and diff checks passed. The filter cell explicitly uses text-left/font-normal and the input uses block display with py-1 instead of py-2 (8px less total vertical padding). Input width, focus styles, localization, and event handlers are unchanged. Specdev reports zero errors and 12 existing warnings. Visual review remains human-owned.
- Mass-heading correction verified on 2026-10-07: all 44 table tests, formatting, split SSR/hydration/Tailwind build, and diff checks passed. Compact headings are Mass/质量/質量; full mass distinction, units, and provenance text remain in existing tooltip descriptions. Specdev reports zero errors and 12 existing warnings. Visual review remains human-owned.
- Verified on 2026-10-07: all 44 focused table tests passed; formatting and split SSR/hydration/Tailwind build passed. Diff checks passed; specdev reports zero errors and 12 existing warnings. The build reports the existing proc-macro-error2 future-compatibility warning.
- The renderer uses the same abbreviation mapping for displayed text and tooltip eligibility, exposing full original text for every mapped value without a native tooltip delay. English heading copy alone changes; Chinese/Japanese headings retain their existing labels. Visual hover/focus review remains human-owned.

## Review

- User visually approved the corrected table on 2026-10-07 (“ok looks good!”). Parked pending the shared batch merge; detailed interaction/locale manual-check boxes remain human-owned and do not imply unreported checks.
- Filter correction reviewed against the screenshot: two class-list changes align the input to the existing 12px cell inset, remove inherited bold text, and reduce height. Shared rendering applies the fix to both catalogs. No filter or navigation behavior changes.
- Requested correction reviewed and verified: shorten localized mass headings and replace the previous heading-level M sin i assertion with compact-label expectations; existing tooltip assertions continue to require M sin i and pl_bmassprov in every locale. Raw fields, values, and tooltip copy are unchanged.
- Reviewed the scoped implementation: only the method display mapping, tooltip eligibility, English heading, and focused regression assertions changed. Other default values retain their existing formatting. Original method strings remain in the row payload; filtering, sorting, and export paths are unchanged. Ready for visual review with `cargo leptos watch --split` at `/exoplanets`.

## Summary

Shorten discovery-method and localized mass headings and wide discovery-method values, retain immediate explanatory tooltips, and align the compact filter input with the first column.

## Log

- 2026-10-07 created: catalog default value abbreviations
- 2026-10-07 scope add src/table/table.rs — Requested discovery-method display abbreviations, full-text tooltips, and compact English heading
- 2026-10-07 scope add locales/en.json — Requested discovery-method display abbreviations, full-text tooltips, and compact English heading
- 2026-10-07 advance draft → ready
- 2026-10-07 scope approved: src/table/table.rs, locales/en.json
- 2026-10-07 advance ready → in-progress/implement
- 2026-10-07 advance in-progress/implement → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 scope add locales/zh-CN.json — User requests compact Mass heading; keep equivalent headings compact in each locale
- 2026-10-07 scope add locales/ja.json — User requests compact Mass heading; keep equivalent headings compact in each locale
- 2026-10-07 advance approval → in-progress/fix (attempts 1)
- 2026-10-07 advance in-progress/fix → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 advance approval → in-progress/fix (attempts 2)
- 2026-10-07 advance in-progress/fix → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 blocked: User visually approved headings and filter layout; awaiting shared batch PR merge
