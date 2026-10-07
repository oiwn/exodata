---
id: 0009-homepage-planet-title-glyph
status: blocked
scope: [src/components/overview.rs, locales/*.json, src/table/table.rs]
created: 2026-10-05
source: https://github.com/oiwn/exodata/issues/154
blocked_reason: User visually approved homepage planet icon; awaiting shared batch PR merge
---
# Task: homepage planet title glyph

## Plan

- [x] Replace the galaxy glyph with a decorative ringed-planet Unicode icon outside the gradient text span; retain the translated title text in all three locales.
- [x] Run formatting and the split SSR/hydration build, then review the diff and hand over homepage visual checks.

## Acceptance

- [x] `cargo fmt --all` and `cargo leptos build --split` pass.
- [x] All three locale hero_title strings retain their title text without the old glyph; the decorative 🪐 span is outside the transparent gradient text span.
- [x] `git diff --check` and `specdev check` pass without task-specific warnings.

## Manual checks

- [ ] At `http://127.0.0.1:3000/`, `/zh-CN`, and `/ja`, confirm the planet icon renders without a square, title translations and gradient remain legible, and the heading fits at desktop/mobile widths.

## Context

- User authorized starting this task on 2026-10-07 after visually approving task 0013. Issue #154 specifically suggests a planet Unicode symbol; use 🪐 outside the transparent gradient span. The localized title text remains unchanged apart from removing its embedded decorative glyph. The icon is hidden from assistive technology.
- Actual edits: `src/components/overview.rs` and three locale hero_title strings. `src/table/table.rs` and existing locale edits remain in scope only to retain the approved uncommitted batch; do not change table behavior in this task.
- Source inspection confirms the existing h1 applies text-transparent/bg-clip-text to the entire title including 🌌. Native browser inspection did not reach the local page; rendered glyph and responsive layout checks remain with the user after compilation.

## Findings

- Verified on 2026-10-07: formatting and split SSR/hydration/Tailwind build passed; all three JSON title strings parse and match their original translated text without the galaxy prefix. Diff check passed; specdev reports zero errors and 9 existing warnings. The existing proc-macro-error2 future-compatibility warning remains. No behavior test was added for this small rendering/copy change.
- The issue body confirms the square-glyph report and requests a proper planet Unicode character. All three locale hero_title strings embed the same 🌌 prefix.

## Summary

Replace the homepage galaxy glyph with a decorative ringed planet outside the gradient title text in all locales.

## Review

- User visually approved the homepage result on 2026-10-07 (“wow looks amazing!”); parked pending the shared batch merge. Manual checklist boxes remain human-owned and do not imply unreported responsive/locale checks.
- Scoped diff reviewed: four implementation files change only title glyph placement and locale title prefixes. The h1 keeps its sizing, pulse animation, and gradient text, while the decorative planet uses normal text color and aria-hidden. No dependencies, image assets, counters, or navigation changes. Run `cargo leptos watch --split` and check `/`, `/zh-CN`, and `/ja`; actual browser glyph rendering and responsive layout remain unverified.

## Log

- 2026-10-05 created: homepage planet title glyph
- 2026-10-05 set source https://github.com/oiwn/exodata/issues/154
- 2026-10-07 scope add src/components/overview.rs — Homepage Unicode planet icon with separate gradient text; retain approved uncommitted table batch changes
- 2026-10-07 scope add locales/*.json — Homepage Unicode planet icon with separate gradient text; retain approved uncommitted table batch changes
- 2026-10-07 scope add src/table/table.rs — Homepage Unicode planet icon with separate gradient text; retain approved uncommitted table batch changes
- 2026-10-07 advance draft → ready
- 2026-10-07 scope approved: src/components/overview.rs, locales/*.json, src/table/table.rs
- 2026-10-07 advance ready → in-progress/implement
- 2026-10-07 advance in-progress/implement → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 blocked: User visually approved homepage planet icon; awaiting shared batch PR merge
