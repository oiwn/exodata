# Current Task Context

Latest task: [0008-exoplanet-table-localized-headings](tasks/0008-exoplanet-table-localized-headings.md) — implemented, verified, and visually approved on 2026-10-07; parked pending the shared batch PR merge.

## Next

Start [0009-homepage-planet-title-glyph](tasks/0009-homepage-planet-title-glyph.md), currently draft. Issue #154 reports the homepage's `🌌 Exoplanet Archive` glyph rendering as a square; inspect the three localized hero-title strings and rendered glyph before defining the small fix.

## Session handoff

- Tasks 0001-0008 have their implementations parked for the shared batch merge; review evidence and remaining human-owned manual checks live in each task file. Do not mark tasks done/archive them before merge or infer unreported manual checks.
- Task 0008 preserves scientific keys/custom headings, translates the eight default headings and shared controls, keeps locale during navigation, tightens horizontal spacing, fixes title descenders, and uses immediate tooltips plus display-only TTV and English Disc. year labels.
- Latest verification: 44 focused table tests, split SSR/hydration/Tailwind build, formatting, and diff checks passed. `specdev check`: zero errors; 25 existing warnings. No new build is needed for these bookkeeping edits.
- Continue remaining tasks before deployment. Git staging/commits and the shared batch merge remain user-managed.
