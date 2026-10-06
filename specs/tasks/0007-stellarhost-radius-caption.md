---
id: 0007-stellarhost-radius-caption
status: blocked
scope: [style/components/*.css, src/components/**/*.rs, src/server/*.rs, locales/*.json, docs/mcp.md]
created: 2026-10-05
source: https://github.com/oiwn/exodata/issues/152
blocked_reason: Caption change verified and approved by the user; further manual check explicitly waived; awaiting shared batch PR merge
---
# Task: stellarhost radius caption

## Plan

- [x] Replace only the bullet separator in the current adopted radius caption with parentheses around the scaling remark.
- [x] Verify the exact wording and scoped diff/spec checks; record the user's advance approval without requiring another visual check.

## Acceptance

- [x] Caption reads `Current adopted radius: {value} R☉ (circles scaled linearly by radius)` with radius formatting and scale behavior unchanged.
- [x] `git diff --check` and `specdev check` pass without new task-specific warnings.

## Summary

Use parentheses for the stellar radius comparison caption's scaling remark.

## Context

- [Issue #152](https://github.com/oiwn/exodata/issues/152) requests replacing the bullet following the solar-radius glyph with parentheses. User authorized the exact before/after wording on 2026-10-07 and explicitly waived a further manual check.
- New implementation is one string change in `src/components/stellarhost_detail/comparison.rs`. Grouped scopes retain existing uncommitted batch changes; no other matching files are new edit scope. No build or new tests are needed for this literal-only presentation change.

## Review

- Exact caption checked on 2026-10-07; only the literal's separator/parentheses changed. Radius calculation and formatting are untouched. Diff/spec checks passed. User approved the exact wording in advance and explicitly waived another visual check; no new test or build was needed.

## Log

- 2026-10-05 created: stellarhost radius caption
- 2026-10-05 set source https://github.com/oiwn/exodata/issues/152
- 2026-10-07 scope add style/components/*.css — One caption literal change plus retained authorized batch changes; intended file recorded in Context
- 2026-10-07 scope add src/components/**/*.rs — One caption literal change plus retained authorized batch changes; intended file recorded in Context
- 2026-10-07 scope add src/server/*.rs — One caption literal change plus retained authorized batch changes; intended file recorded in Context
- 2026-10-07 scope add locales/*.json — One caption literal change plus retained authorized batch changes; intended file recorded in Context
- 2026-10-07 scope add docs/mcp.md — One caption literal change plus retained authorized batch changes; intended file recorded in Context
- 2026-10-07 advance draft → ready
- 2026-10-07 scope approved: style/components/*.css, src/components/**/*.rs, src/server/*.rs, locales/*.json, docs/mcp.md
- 2026-10-07 advance ready → in-progress/implement
- 2026-10-07 advance in-progress/implement → in-progress/verify
- 2026-10-07 advance in-progress/verify → in-progress/review
- 2026-10-07 advance in-progress/review → approval
- 2026-10-07 blocked: Caption change verified and approved by the user; further manual check explicitly waived; awaiting shared batch PR merge
