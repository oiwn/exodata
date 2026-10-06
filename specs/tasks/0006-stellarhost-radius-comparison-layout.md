---
id: 0006-stellarhost-radius-comparison-layout
status: blocked
scope: [style/components/*.css, src/components/**/*.rs, src/server/*.rs, locales/*.json, docs/mcp.md]
created: 2026-10-05
source: https://github.com/oiwn/exodata/issues/151
blocked_reason: Implementation verified and visually approved by the user; awaiting shared batch PR merge
---
# Task: stellarhost radius comparison layout

## Plan

- [x] Confirm the requested single-panel comparison appearance, declare focused scope, and document the visual contract.
- [x] Remove per-star card backgrounds, borders, and rounded corners; preserve the shared dark comparison panel, radius scaling, labels, alignment, spacing, and mobile/desktop arrangement.
- [x] Run the split SSR/hydration build and scoped diff/spec checks; review the CSS-only fix and stop for manual inspection.

## Acceptance

- [x] `cargo leptos build --split` passes for SSR/hydration and Tailwind.
- [x] The per-star comparison selector has no background, border, or rounded-corner styling; the outer panel retains its dark background/frame and responsive grid. Comparison Rust/scale logic is unchanged.
- [x] `git diff --check` and `specdev check` pass without new task-specific warnings.

## Manual checks

- [ ] Run `cargo leptos watch --split`; open `http://127.0.0.1:3000/stellarhosts/Kepler-154` and inspect Radius against the Sun: both stars appear within one shared dark panel without separate visible cards.
- [ ] Check a smaller star such as TRAPPIST-1 and a larger-radius host available in the loaded catalog; confirm circle sizes/labels remain aligned. Check side-by-side desktop and stacked mobile layout for overflow and readable captions.

## Summary

Show the stellar radius comparison in one shared dark panel by removing the individual star card decoration.

## Context

- [Issue #151](https://github.com/oiwn/exodata/issues/151) requests removing the inner boxes around comparison stars while retaining the darker outer container. User authorized the next task on 2026-10-07.
- New implementation is limited to `style/components/stellarhost-detail.css`; alignment wrappers remain to group each star with its labels, but have no visible card frame/background. No comparison scale, diameter limits, caption wording, or data selection changes are included.
- Grouped scope globs cover retained uncommitted task 0002-0005 changes for batch validation. Other matched files are outside the new implementation plan. Task 0007 owns the radius caption follow-up.

## Findings

- Verification on 2026-10-07: split SSR/hydration/Tailwind build and `git diff --check` passed. A focused CSS check confirmed the inner decorations are gone and the shared panel/responsive grid remain; comparison Rust has no diff. Specdev reports zero errors and 31 existing warnings. Visual checks remain with the user; no additional tests were added for this one-line CSS change.
- `.host-comparison` already provides the shared dark panel and responsive grid. `.host-comparison__card` adds a second rounded border and translucent background around each star; removing only that decoration addresses the issue without changing layout mechanics.

## Review

- User visually reviewed the comparison on 2026-10-07, confirmed it looks great, and requested marking it reviewed. Implementation and visual approval are complete; park pending the shared batch merge rather than archiving before merge. Manual checklist boxes remain human-owned.
- Reviewed the one-line CSS change: removed only `rounded-[1.25rem]`, `border`, `border-white/8`, and `bg-white/4` from the per-star wrapper. Shared outer panel, alignment, minimum height, padding, text, responsive layout, and scale computation are retained. Ready for manual review.

## Log

- 2026-10-05 created: stellarhost radius comparison layout
- 2026-10-05 set source https://github.com/oiwn/exodata/issues/151
- 2026-10-07 scope add style/components/*.css — Single-panel comparison CSS fix plus retained authorized batch changes; exact new file limit recorded in Context
- 2026-10-07 scope add src/components/**/*.rs — Single-panel comparison CSS fix plus retained authorized batch changes; exact new file limit recorded in Context
- 2026-10-07 scope add src/server/*.rs — Single-panel comparison CSS fix plus retained authorized batch changes; exact new file limit recorded in Context
- 2026-10-07 scope add locales/*.json — Single-panel comparison CSS fix plus retained authorized batch changes; exact new file limit recorded in Context
- 2026-10-07 scope add docs/mcp.md — Single-panel comparison CSS fix plus retained authorized batch changes; exact new file limit recorded in Context
- 2026-10-07 advance draft → ready
- 2026-10-07 scope approved: style/components/*.css, src/components/**/*.rs, src/server/*.rs, locales/*.json, docs/mcp.md
- 2026-10-07 advance ready → in-progress/implement
- 2026-10-07 advance in-progress/implement → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 blocked: Implementation verified and visually approved by the user; awaiting shared batch PR merge
