---
id: 0011-homepage-counter-links
status: approval
scope: [src/components/overview.rs]
created: 2026-10-05
source: https://github.com/oiwn/exodata/issues/156
---
# Task: homepage counter links

## Plan

- [x] Make the full Stellar Systems and Exoplanets counter cards semantic links to their locale-aware catalog routes; preserve visuals and card height, and add visible keyboard focus.
- [x] Verify formatting, split SSR/hydration build, rendered links in all locales, and workspace Clippy; hand over click/keyboard checks.

## Acceptance

- [x] `cargo fmt --all` and `cargo leptos build --split` pass.
- [x] Live SSR checks confirm the two complete cards link to the matching catalog in English, Chinese, and Japanese; average cards remain non-links.
- [x] `cargo lx --locked -- -D warnings`, `git diff --check`, and `specdev check` pass without task-specific warnings.

## Manual checks

- [ ] On `/`, `/zh-CN`, and `/ja`, click the Stellar Systems and Exoplanets cards, including their blank padding areas, and confirm the matching table opens in the same locale. Tab to each card, confirm a visible focus ring, and use Enter to navigate. Check desktop/mobile layout and existing hover animation.

## Context

- User authorized task 0011 on 2026-10-08 after committing the preceding batch. Issue #156 requests whole-box links because the two count cards currently animate on hover but clicking does nothing. Link Stellar Systems to `/stellarhosts` and Exoplanets to `/exoplanets`, preserving locale via the existing helper and using Leptos A for client navigation.
- The two average-value cards have no requested destination and retain existing behavior. Do not change metrics, localized copy, gradients, hover animation, or other homepage sections. Preserve grid/card sizing after adding the link wrappers.

## Summary

Make the homepage Stellar Systems and Exoplanets counter cards clickable links to their locale-aware catalogs, with visible keyboard focus.

## Findings

- Verified on 2026-10-08: formatting, split SSR/hydration/Tailwind build, workspace/all-targets/all-features Clippy with warnings denied, and diff checks passed. Live SSR checks across `/`, `/zh-CN`, and `/ja` found exactly two complete counter-card anchors per page with matching localized catalog destinations and focus-ring classes; all four cards render, with the two averages remaining non-links. Specdev reports zero errors and 3 existing warnings. The existing proc-macro-error2 future-compatibility warning remains informational.
- No new tests or abstractions were added for the small reversible link change. Browser clicks, Tab/Enter navigation, hover/focus appearance, and desktop/mobile visual layout remain human-owned checks.

## Review

- Reviewed the one-file implementation against the plan: two Leptos A wrappers make the entire existing cards clickable, use localized_path, and add focus-visible rings. Shared card h-full preserves grid height after wrapping. Metric values, localized labels, gradients, existing hover animation, average-card behavior, and other homepage sections are unchanged. Ready for browser review with `cargo leptos watch --split` at the three homepage routes.

## Log

- 2026-10-05 created: homepage counter links
- 2026-10-05 set source https://github.com/oiwn/exodata/issues/156
- 2026-10-08 scope add src/components/overview.rs — Whole-card locale-aware links for the two catalog counters with preserved layout and keyboard focus styling
- 2026-10-08 advance draft → ready
- 2026-10-08 scope approved: src/components/overview.rs
- 2026-10-08 advance ready → in-progress/implement
- 2026-10-08 advance in-progress/implement → in-progress/verify
- 2026-10-08 advance in-progress/verify → in-progress/review
- 2026-10-08 advance in-progress/review → approval
