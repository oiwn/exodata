---
id: 0005-stellarhost-source-text-contrast
status: blocked
scope: [style/components/*.css, src/components/**/*.rs, src/server/*.rs, locales/*.json, docs/mcp.md]
created: 2026-10-05
source: https://github.com/oiwn/exodata/issues/150
blocked_reason: Source-text contrast implementation verified; awaiting manual inspection and shared batch merge while task 0006 proceeds
---
# Task: stellarhost source text contrast

## Plan

- [x] Confirm the source paragraph's missing explicit text color, document its intended contrast, and declare focused implementation scope.
- [x] Add a semantic light-text class to the Stellar source/System source paragraph and the no-source-row message; preserve reference link colors, destinations, text, and record selection.
- [x] Run Rust formatting, the split SSR/hydration build, and diff/spec checks; review the focused change and stop for manual inspection.

## Acceptance

- [x] `cargo fmt --all` and `cargo leptos build --split` pass.
- [x] Both source paragraph branches use the dedicated semantic class; its CSS sets `text-slate-300`, while the reference-link component/style is unchanged.
- [x] `git diff --check` and `specdev check` pass without new task-specific warnings.

## Manual checks

- [ ] Run `cargo leptos watch --split`; open `http://127.0.0.1:3000/stellarhosts/Kepler-154` and inspect Canonical Summary. Confirm Stellar source, System source, and the separator are readable against the dark background while reference links retain their blue styling.
- [ ] Inspect the paragraph at mobile width and follow a reference link to confirm the destination and behavior are unchanged. If a host without a usable source row is available, check its empty-source message too.

## Summary

Make stellar-host summary source labels and the empty-source message readable against the dark detail background.

## Context

- [Issue #150](https://github.com/oiwn/exodata/issues/150) reports black Stellar source text on a dark background. User authorized the next task on 2026-10-07.
- New edits are limited to `src/components/stellarhost_detail/summary.rs` and `style/components/stellarhost-detail.css`, with the technical contract recorded in `specs/web-frontend.md`.
- Grouped scope globs retain uncommitted work from tasks 0002-0004 for batch checking. Other matched files are not new implementation scope; preserve existing guide, homepage, palette, sitemap, and locale changes.

## Findings

- Verification on 2026-10-07: formatting, split SSR/hydration/Tailwind build, and `git diff --check` passed. A focused source/CSS check confirmed both paragraph branches use `host-detail-section__source` with `text-slate-300`. Specdev reports zero errors and 34 existing warnings, none for this task. Rendered inspection remains in the user's manual checklist; no additional tests or browser automation were added for this presentation-only change.
- The selected-row source paragraph and no-source fallback are bare `<p>` elements. `ProvenanceCell` styles links as light blue, but labels/separator/plain fallback values inherit the page's default text color. A dedicated paragraph class gives all non-link source text an explicit light color.

## Review

- Reviewed the two-file implementation: only source paragraph classes and their dedicated text-color selector were added. The labels, separator, reference values, reference-link renderer/styles, selected-row behavior, and typography are preserved. Ready for the user's manual contrast check after the successful split build.

## Log

- 2026-10-05 created: stellarhost source text contrast
- 2026-10-05 set source https://github.com/oiwn/exodata/issues/150
- 2026-10-07 scope add style/components/*.css — Focused summary source contrast fix plus retained authorized batch changes; intended edits recorded in Context
- 2026-10-07 scope add src/components/**/*.rs — Focused summary source contrast fix plus retained authorized batch changes; intended edits recorded in Context
- 2026-10-07 scope add src/server/*.rs — Focused summary source contrast fix plus retained authorized batch changes; intended edits recorded in Context
- 2026-10-07 scope add locales/*.json — Focused summary source contrast fix plus retained authorized batch changes; intended edits recorded in Context
- 2026-10-07 scope add docs/mcp.md — Focused summary source contrast fix plus retained authorized batch changes; intended edits recorded in Context
- 2026-10-07 advance draft → ready
- 2026-10-07 scope approved: style/components/*.css, src/components/**/*.rs, src/server/*.rs, locales/*.json, docs/mcp.md
- 2026-10-07 advance ready → in-progress/implement
- 2026-10-07 advance in-progress/implement → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 blocked: Source-text contrast implementation verified; awaiting manual inspection and shared batch merge while task 0006 proceeds
