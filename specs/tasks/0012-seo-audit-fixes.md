---
id: 0012-seo-audit-fixes
status: draft
scope: []
created: 2026-10-06
source: https://github.com/oiwn/exodata/issues/159
---
# Task: seo audit fixes

## Plan

- [ ] Review [the existing SEO audit](../exodata_seo_audit.md), reproduce its findings against current code and the deployed site, and record dated evidence plus a confirmed, resolved, unconfirmed, or inaccurate classification for each audit item.
- [ ] Map confirmed findings to existing tasks and technical specs, including the MCP discovery work in task 0002 and missing-detail errors in task 0010; avoid implementing the same fix in multiple tasks.
- [ ] Agree on a dependency-ordered, commit-sized implementation scope and exact acceptance commands before advancing. If the confirmed work spans multiple commits, obtain an approved task breakdown rather than treating the entire audit as one small change.
- [ ] Update the relevant technical contracts before behavior changes, declare implementation files through specdev, and implement only the approved fixes with focused regression coverage.
- [ ] Verify the approved URL, metadata, sitemap, content, or performance changes using the matching focused checks and fastest meaningful browser inspection; record results, remaining findings, and concrete follow-up ownership before review.

## Acceptance

- [ ] `specdev check` reports no errors or task-specific quality warnings after scope is declared, and scoped `git diff --check` passes.
- [ ] Every audit item has a dated evidence-based disposition. Original scores, page counts, timing measurements, and ranking predictions are not treated as current facts or acceptance thresholds.
- [ ] The reviewed scope names the findings being fixed, their dependencies, exact implementation files, and runnable acceptance commands. Confirmed findings outside that scope have explicit follow-up ownership; no issue is silently dropped.
- [ ] For approved routing fixes, regression checks cover existing records, nonexistent records, encoded names, URL variants, trailing slashes, and locale routes, with expected status, redirect, and canonical behavior specified before implementation.
- [ ] For approved metadata or sitemap fixes, rendered HTML and sitemap checks prove that the intended title, description, language, canonical, structured-data URLs, and indexed route targets agree. Existing public URL contracts are preserved unless a migration is explicitly approved.
- [ ] For approved performance or infrastructure changes, record repeatable before/after measurements and deployment-specific validation. Run only the focused Rust, SSR/hydration, browser, or deployment checks required by the approved scope; write their exact commands before moving to ready.

## Context

- Source: [issue #159, SEO audit fixes](https://github.com/oiwn/exodata/issues/159). Its body says Improve SEO, a lot of small tasks; no comments or detailed acceptance criteria were present on 2026-10-06.
- Primary input: [specs/exodata_seo_audit.md](../exodata_seo_audit.md), a specialist-agent audit dated 2026-09-18. Preserve the original report; record current reproduction results and decisions in this task's Findings rather than rewriting its historical observations.
- This draft tracks the requested audit-fix work. The report covers several independent subsystems and is not yet a commit-sized implementation specification. Keep it in draft until the user approves the sizing and concrete fix scope or a breakdown.
- File scope remains empty pending review. Creating this task does not authorize website changes, external account access, directory submissions, deployment, or an unreviewed URL migration.

## Findings

- The audit covers C1-C4 (missing-record status, canonical normalization, malformed/trailing-slash handling, locale canonicals), H1-H5 (hreflang, security headers, detail prose, descriptions, mobile payload), M1-M5 (Dataset metadata, faceted URL crawling, provenance, IndexNow, project identity), and L1-L5 (language/description duplication, text-route fallbacks, llms.txt, documentation metadata).
- Repository review found `/docs/mcp` absent from `build_static_urls` in `src/server/handlers.rs`. Verify the deployed sitemap before fixing it; sitemap omission alone does not establish that a page is unindexed.
- Some audit conclusions need qualification: docs already emit a title, description, and canonical in `src/components/docs/page.rs`, and `/about` redirects to the documentation overview. Assess metadata specificity and missing project information rather than assuming these surfaces do not exist.
- The audit's production 502 observations, redirect behavior, duplicate rendered tags, and Lighthouse measurements have not been reproduced in this task. Repository evidence alone does not prove current deployment behavior.
- The audit recommends route normalization and new hyphenated slugs. A new URL scheme, case-folding policy, or redirects can change public lookup behavior; specify collision handling and backward compatibility, and obtain approval before a migration.
- Blanket query-URL blocking is not an automatic fix: [Google's robots.txt guidance](https://developers.google.com/search/docs/crawling-indexing/robots/intro) distinguishes crawling from indexing. Define the desired indexing and canonical policy before editing robots directives.
- Structured data must describe actual content and provenance. [Google's structured-data guidance](https://developers.google.com/search/docs/appearance/structured-data/sd-policies) does not guarantee rich results, and the audit's score is not a Google metric.

## Coordination

- User-approved implementation extension on 2026-10-06: task 0002 now owns MCP guide metadata/setup and local `/docs/mcp` sitemap inclusion. Do not duplicate those changes here; production crawler access, deployed sitemap inspection, and other audit items remain deferred.
- [Task 0002](0002-exoplanets-mcp-seo-research.md), issue #146: MCP keyword research, discovery baseline, evidence, and implementation recommendations. This task owns reviewed technical SEO fixes, including an MCP sitemap fix if reproduced and selected.
- [Task 0003](0003-homepage-mcp-description.md), issue #148: homepage MCP messaging. Avoid duplicating homepage copy changes here without an explicit scope decision.
- [Task 0010](0010-catalog-detail-error-pages.md), issue #155: branded detail errors and appropriate status codes, overlapping audit C1. Coordinate missing-record behavior and tests rather than implementing conflicting fixes.
- Dependency ordering is established after reproduction. The audit's proposed sequence is input for review, not an approved implementation plan.

## Log

- 2026-10-06 created: seo audit fixes
- 2026-10-06 set source https://github.com/oiwn/exodata/issues/159
