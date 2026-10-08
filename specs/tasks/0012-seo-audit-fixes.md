---
id: 0012-seo-audit-fixes
status: blocked
scope: [src/*.rs, src/components/*_detail/page.rs, src/server/*.rs, end2end/tests/smoke.spec.ts, src/table/table.rs, src/components/insights/common.rs]
created: 2026-10-06
source: https://github.com/oiwn/exodata/issues/159
blocked_reason: User visually approved the verified first-five SEO fix batch; awaiting user-managed commit/shared batch merge
---
# Task: seo audit fixes

## Plan

- [x] Review [the existing SEO audit](../exodata_seo_audit.md), reproduce its findings against current code and the deployed site, and record dated evidence plus a confirmed, resolved, unconfirmed, or inaccurate classification for each audit item.
- [x] Map confirmed findings to existing tasks and technical specs, including the MCP discovery work in task 0002 and missing-detail errors in task 0010; avoid implementing the same fix in multiple tasks.
- [x] Agree on a dependency-ordered, commit-sized implementation scope and exact acceptance commands before advancing. If the confirmed work spans multiple commits, obtain an approved task breakdown rather than treating the entire audit as one small change.
- [x] Update the relevant technical contracts before behavior changes, declare implementation files through specdev, and implement only the approved fixes with focused regression coverage.
- [x] Verify the approved URL, metadata, sitemap, content, or performance changes using the matching focused checks and fastest meaningful browser inspection; record results, remaining findings, and concrete follow-up ownership before review.

## Acceptance

- [x] `specdev check` reports no errors or task-specific quality warnings after scope is declared, and scoped `git diff --check` passes.
- [x] Every audit item has a dated evidence-based disposition. Original scores, page counts, timing measurements, and ranking predictions are not treated as current facts or acceptance thresholds.
- [x] The reviewed scope names the findings being fixed, their dependencies, exact implementation files, and runnable acceptance commands. Confirmed findings outside that scope have explicit follow-up ownership; no issue is silently dropped.
- [x] For approved routing fixes, regression checks cover existing records, nonexistent records, encoded names, URL variants, trailing slashes, and locale routes, with expected status, redirect, and canonical behavior specified before implementation.
- [x] For approved metadata or sitemap fixes, rendered HTML and sitemap checks prove that the intended title, description, language, canonical, structured-data URLs, and indexed route targets agree. Existing public URL contracts are preserved unless a migration is explicitly approved.
- [x] `cargo fmt --all`, `cargo leptos build --split`, `cargo test --locked --workspace --all-features`, and `cargo lx --locked -- -D warnings` pass. Focused routing/SSR checks and `cd end2end && npx playwright test --grep "SEO detail"` cover the new behavior. No performance/infrastructure change is included in this approved batch.

## Manual checks

- [ ] Check valid/missing host and planet pages, trailing-slash redirects, and success → missing → success client navigation at `/`, `/zh-CN`, and `/ja`. Inspect one lang attribute, one successful description/canonical, and no profile metadata on errors. Verify successful profile visuals and the existing branded error layout.

## Context

- Source: [issue #159, SEO audit fixes](https://github.com/oiwn/exodata/issues/159). Its body says Improve SEO, a lot of small tasks; no comments or detailed acceptance criteria were present on 2026-10-06.
- Primary input: [specs/exodata_seo_audit.md](../exodata_seo_audit.md), a specialist-agent audit dated 2026-09-18. Preserve the original report; record current reproduction results and decisions in this task's Findings rather than rewriting its historical observations.
- User approved implementation on 2026-10-08 of the first five fixes as one batch: safe detail routing, consistent existing-URL encoding, single description tags, one route-selected HTML lang attribute, and no fabricated profile metadata on errors. No deployment, slug migration, case folding, or localized indexing changes.
- Exact edits: `src/app.rs`, `src/main.rs`, `src/metadata_helpers.rs`, `src/structured_data.rs`, both detail `page.rs` files, `src/server.rs`, `src/server/handlers.rs`, new `src/server/detail_routes.rs`, `src/server/tests.rs`, `src/table/table.rs`, and `end2end/tests/smoke.spec.ts`. Existing URL expectations in `src/server/mcp.rs` and `src/components/insights/common.rs` change only to match the shared encoder. Grouped scope does not authorize unrelated edits.
- GET/HEAD website detail paths, including locale prefixes: redirect trailing slashes 301 to the slash-free path while preserving query; invalid percent escapes/UTF-8 return 400; extra literal segments return 404; valid missing names retain branded 404. REST/MCP/assets/exports and non-GET/HEAD requests pass unchanged.
- Share RFC3986 path-segment encoding (retain - . _ ~; spaces %20; encode reserved characters) across generated links/canonicals/JSON-LD/sitemaps. Preserve exact case and literal + lookup; legacy encoding variants remain usable. Successful profile metadata comes from the returned record name only. Errors retain generic title/noindex and no profile canonical/description/Dataset schema. Localized detail canonicals remain English until translation is complete.

## Findings

- Implementation verification on 2026-10-08: split SSR/hydration/Tailwind build and formatting passed; full locked workspace/all-features tests passed (156 web tests), and final workspace/all-targets/all-features Clippy passed with warnings denied. TypeScript typecheck and two focused Playwright SEO detail tests passed against the running local server, including SSR head counts, locale lang, canonical/JSON-LD/sitemap agreement, HTTP GET/HEAD redirects, invalid paths, legacy encodings, and success → missing → success title/tag cleanup in all three locales. Existing proc-macro-error2 future-compatibility warning remains informational. Diff/spec checks passed with zero errors and 2 existing global warnings.
- The in-process full-page SSR harness was removed before completion because Polars blocking queries are unsupported inside a Tokio LocalSet. Actual full-document responses are asserted in Playwright instead; middleware and encoder regressions remain Rust tests. Production was not deployed or used as validation of these local fixes.
- Recheck 2026-10-08 (local current code versus undeployed production): **C1** status fixed locally by 0010; remaining fake error canonical/description addressed here. **C2** encoding inconsistency confirmed; case/+ variants are failed lookups, not successful duplicate records. **C3** trailing-slash failure confirmed remotely (502) and locally (closed response); encoded script-like input is a missing record, invalid percent input returns 400 at the edge. **C4/H1** English detail canonicals/no hreflang/localized sitemap absence confirmed but intentionally deferred by localization spec; no indexing policy change here.
- Recheck 2026-10-08: **H2** security headers absent remotely and in nginx templates; follow-up outside this batch. **H3** planet prose absent, but ranking/thin-content claims are unverified; prose scope needs separate approval. **H4** uniform-description claim overstated: factual descriptions exist alongside duplicate fallbacks; duplicate tags fixed here. **H5** duplicate metadata confirmed (45KB global dictionary plus 33KB decoded detail metadata in a sampled resource); the 87% assertion includes 96KB of actual records. Mobile LCP not remeasured; performance follow-up requires fresh evidence.
- Recheck 2026-10-08: **M1** minimal Dataset schema confirmed, provenance/download enrichment follow-up. **M2** faceted URLs canonicalize to the base table already; blanket robots blocking is not approved. **M3** named source-reference tables exist, so no-provenance claim is overstated; explicit NASA/freshness/citation copy remains a follow-up. **M4** IndexNow absent, optional external submission work requires separate scope. **M5** docs/About content exists; maintainer/Organization enrichment is a separate content decision.
- Recheck 2026-10-08: **L1/L2** conflicting lang attributes and duplicate detail descriptions confirmed locally/remotely and included here. **L3** absent sitemap.xml/llms.txt routes return large HTML 404 bodies remotely; lightweight text fallback is deferred. **L4** llms.txt absent, optional follow-up. **L5** docs already emit title/description/canonical; specificity/schema improvements remain a follow-up. MCP sitemap inclusion is already fixed locally by 0002; production still omits it because the batch is not deployed. Historical audit claims below are preserved as earlier observations.

- The audit covers C1-C4 (missing-record status, canonical normalization, malformed/trailing-slash handling, locale canonicals), H1-H5 (hreflang, security headers, detail prose, descriptions, mobile payload), M1-M5 (Dataset metadata, faceted URL crawling, provenance, IndexNow, project identity), and L1-L5 (language/description duplication, text-route fallbacks, llms.txt, documentation metadata).
- Repository review found `/docs/mcp` absent from `build_static_urls` in `src/server/handlers.rs`. Verify the deployed sitemap before fixing it; sitemap omission alone does not establish that a page is unindexed.
- Some audit conclusions need qualification: docs already emit a title, description, and canonical in `src/components/docs/page.rs`, and `/about` redirects to the documentation overview. Assess metadata specificity and missing project information rather than assuming these surfaces do not exist.
- The audit's production 502 observations, redirect behavior, duplicate rendered tags, and Lighthouse measurements have not been reproduced in this task. Repository evidence alone does not prove current deployment behavior.
- The audit recommends route normalization and new hyphenated slugs. A new URL scheme, case-folding policy, or redirects can change public lookup behavior; specify collision handling and backward compatibility, and obtain approval before a migration.
- Blanket query-URL blocking is not an automatic fix: [Google's robots.txt guidance](https://developers.google.com/search/docs/crawling-indexing/robots/intro) distinguishes crawling from indexing. Define the desired indexing and canonical policy before editing robots directives.
- Structured data must describe actual content and provenance. [Google's structured-data guidance](https://developers.google.com/search/docs/appearance/structured-data/sd-policies) does not guarantee rich results, and the audit's score is not a Google metric.

## Coordination

- Remaining audit work is retained in the dated Findings above. The user owns selecting separate follow-up tasks for headers, payload/performance, schema/provenance, prose/project identity, text-route fallbacks, and optional discovery features. Localization canonicals/hreflang/sitemap eligibility remain owned by the localization specification; no silent indexing-policy expansion is included here.

- User-approved implementation extension on 2026-10-06: task 0002 now owns MCP guide metadata/setup and local `/docs/mcp` sitemap inclusion. Do not duplicate those changes here; production crawler access, deployed sitemap inspection, and other audit items remain deferred.
- [Task 0002](0002-exoplanets-mcp-seo-research.md), issue #146: MCP keyword research, discovery baseline, evidence, and implementation recommendations. This task owns reviewed technical SEO fixes, including an MCP sitemap fix if reproduced and selected.
- [Task 0003](0003-homepage-mcp-description.md), issue #148: homepage MCP messaging. Avoid duplicating homepage copy changes here without an explicit scope decision.
- [Task 0010](0010-catalog-detail-error-pages.md), issue #155: branded detail errors and appropriate status codes, overlapping audit C1. Coordinate missing-record behavior and tests rather than implementing conflicting fixes.
- Dependency ordering is established after reproduction. The audit's proposed sequence is input for review, not an approved implementation plan.

## Review

- User visually approved the first-five-fix batch on 2026-10-08 (“just checked, all looks good!”). Implementation, automated verification, and visual review are complete; park pending the user-managed commit/shared batch merge. Manual checklist boxes remain human-owned rather than implying individually reported checks.
- Scoped diff reviewed against the approved first-five batch. Middleware runs outside the export/Leptos handlers, validates only GET/HEAD website detail paths, preserves raw query/locale on slash redirects, and does not change exact-case/literal-plus lookup or public slug scheme. Shared encoding removes duplicate encoders and hand-encoding; existing MCP/insight URL assertions now match the intended encoding.
- Detail head tags are owned by successful payload branches, so errors and client transitions cannot retain fabricated record metadata. The locale provider is the sole lang owner. Successful profile/error visuals are unchanged. Actual SSR responses and Chromium navigation pass; no further production audit, deployment, staging, commits, or archival were performed.

## Summary

Fix detail trailing-slash/malformed-path handling and URL consistency, remove duplicate description/lang tags, and emit record metadata only for successful profiles.

## Log

- 2026-10-06 created: seo audit fixes
- 2026-10-06 set source https://github.com/oiwn/exodata/issues/159
- 2026-10-08 scope add src/*.rs — Approved detail routing and metadata batch; exact implementation limits recorded in Context
- 2026-10-08 scope add src/components/*_detail/page.rs — Approved detail routing and metadata batch; exact implementation limits recorded in Context
- 2026-10-08 scope add src/server/*.rs — Approved detail routing and metadata batch; exact implementation limits recorded in Context
- 2026-10-08 scope add end2end/tests/smoke.spec.ts — Approved detail routing and metadata batch; exact implementation limits recorded in Context
- 2026-10-08 advance draft → ready
- 2026-10-08 scope approved: src/*.rs, src/components/*_detail/page.rs, src/server/*.rs, end2end/tests/smoke.spec.ts
- 2026-10-08 advance ready → in-progress/implement
- 2026-10-08 scope add src/table/table.rs — Existing table detail links hand-encode names; use the same approved path-segment encoder as other generated links
- 2026-10-08 scope add src/components/insights/common.rs — Update the existing generated-link assertion to the approved shared URL encoding
- 2026-10-08 advance in-progress/implement → in-progress/verify
- 2026-10-08 advance in-progress/verify → in-progress/review
- 2026-10-08 advance in-progress/review → approval
- 2026-10-08 blocked: User visually approved the verified first-five SEO fix batch; awaiting user-managed commit/shared batch merge
