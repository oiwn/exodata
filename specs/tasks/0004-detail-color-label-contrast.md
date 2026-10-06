---
id: 0004-detail-color-label-contrast
status: approval
scope: [style/components/*.css, src/components/**/*.rs, src/server/*.rs, locales/*.json, docs/mcp.md]
created: 2026-10-05
source: https://github.com/oiwn/exodata/issues/149
---
# Task: detail color label contrast

## Plan

- [x] Record the star badge contrast contract and declare the focused files; inspect the corresponding planet badge before deciding whether it needs the same treatment.
- [x] Render both star badge text lines in black over an opaque pale temperature-derived tint so readability does not depend on the star/background underneath; retain badge shape, position, and wording.
- [x] Run existing star-color tests, Rust formatting, and the split SSR/hydration build; review the focused diff and stop for the user's visual check.

## Acceptance

- [x] `cargo test --locked -p exodata-web --lib star_color` passes.
- [x] `cargo fmt --all` and `cargo leptos build --split` pass.
- [x] Palette contrast calculation checks black text against generated opaque badge colors for cool/hot endpoints, intermediate anchors, and missing-temperature fallback; minimum normal-text contrast is 4.5:1.
- [x] `git diff --check` and `specdev check` pass without new task-specific warnings; no unrelated styles or data behavior change.

## Manual checks

- [ ] Run `cargo leptos watch --split` with existing runtime data; open `http://127.0.0.1:3000/stellarhosts/Kepler-154` and confirm both “Approximate color” and “from effective temperature” are readable in black on the pale badge.
- [ ] Inspect a cool star (for example TRAPPIST-1), a hotter star, and a missing-temperature host available in the loaded catalog; check badge contrast and placement at desktop and mobile sizes.
- [ ] Open a planet detail page such as `/exoplanets/Kepler-22%20b` and inspect the existing dark badge. Report any remaining contrast problem before changing that badge's treatment.

## Summary

Improve stellar-host color badge contrast with black text and an opaque pale temperature-derived background.

## Context

- [Issue #149](https://github.com/oiwn/exodata/issues/149) names `/stellarhosts/Kepler-154` and asks for black text in the Approximate color badge; the planet equivalent is an inspection follow-up, not an automatic black-text change.
- User authorized starting on 2026-10-07. Intended new edits: `style/components/stellarhost-detail.css` and `src/components/stellarhost_detail/star_color.rs`; technical contract in `specs/web-frontend.md`.
- Grouped scopes include retained uncommitted task 0002/0003 files solely for batch checking: `docs/mcp.md`, `src/components/docs/registry.rs`, `src/components/homepage_manual.rs`, `src/server/handlers.rs`, `src/server/tests.rs`, `style/components/homepage-manual.css`, and the three locale JSON files. Other files matched by the globs remain outside this plan.

## Findings

- Verification on 2026-10-07: two existing star-color tests passed; Rust formatting and split SSR/hydration/Tailwind build passed. A read-only calculation using the source palette anchors/highlight factor checked 716 samples including interpolation and fallback, with minimum black-text contrast 14.81:1. `git diff --check` passed; specdev reports zero errors and 37 existing warnings, with none for this task. Rendered appearance remains for the user to confirm.
- Star badge labels currently use `text-slate-400` and `text-white`, with a darkened temperature tint at 24% opacity. An opaque pale tint prevents underlying star/glow/background from changing text contrast.
- Planet labels already sit on a dark slate badge; retain its existing treatment pending the user's rendered-page inspection. No local server was listening during initial inspection.

## Review

- Reviewed the focused three-line implementation: the badge uses `rgb_css(highlight)` instead of a translucent dark tint, and both text selectors use `text-black`. No palette anchors, star core/glow, badge geometry, wording, planet styles, or data contracts were changed. Numeric checks establish text/background contrast; the manual checklist covers rendered placement and the planet badge inspection.

## Log

- 2026-10-05 created: detail color label contrast
- 2026-10-05 set source https://github.com/oiwn/exodata/issues/149
- 2026-10-07 scope add style/components/*.css — Focused star badge fix plus retained authorized batch changes; exact file limits recorded in Context
- 2026-10-07 scope add src/components/**/*.rs — Focused star badge fix plus retained authorized batch changes; exact file limits recorded in Context
- 2026-10-07 scope add src/server/*.rs — Focused star badge fix plus retained authorized batch changes; exact file limits recorded in Context
- 2026-10-07 scope add locales/*.json — Focused star badge fix plus retained authorized batch changes; exact file limits recorded in Context
- 2026-10-07 scope add docs/mcp.md — Focused star badge fix plus retained authorized batch changes; exact file limits recorded in Context
- 2026-10-07 advance draft → ready
- 2026-10-07 scope approved: style/components/*.css, src/components/**/*.rs, src/server/*.rs, locales/*.json, docs/mcp.md
- 2026-10-07 advance ready → in-progress/implement
- 2026-10-07 advance in-progress/implement → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
