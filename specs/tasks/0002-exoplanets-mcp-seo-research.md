---
id: 0002-exoplanets-mcp-seo-research
status: blocked
scope: [specs/exoplanets-mcp-seo.md, docs/mcp.md, src/components/docs/registry.rs, src/server/handlers.rs, src/server/tests.rs]
created: 2026-10-05
source: https://github.com/oiwn/exodata/issues/146
attempts: 1
blocked_reason: Guide and sitemap implementation verified; awaiting local manual review and shared batch PR merge while task 0003 proceeds
---
# Task: exoplanets mcp seo research

## Plan

- [x] Review and approve the research boundary and proposed output below; declare scope through specdev before starting the research deliverable.
- [x] Verify Exodata's supported MCP capabilities against the repository and public MCP documentation; inspect the existing SEO audit as dated background, not current evidence.
- [x] Record the current discovery baseline for `/` and `/docs/mcp`: rendered title, description, headings, canonical, robots directives, internal links, sitemap inclusion, and consistency of connection instructions.
- [x] Search the current web for exoplanet MCP and closely related queries, record search dates and source URLs, and inspect relevant provider documentation or repositories to distinguish hosted MCP servers from APIs, local wrappers, and directories.
- [x] Collect 10-20 candidate English keyword phrases grouped by connection/setup, catalog querying, and NASA data access; prioritize a shortlist of five using relevance and observed search intent, with evidence and limitations.
- [x] Write a compact research report mapping the shortlist to existing homepage and MCP documentation routes, proposed titles/descriptions/headings, supported positioning, content gaps, and concrete follow-up recommendations; verify sources and links before review.
- [x] Define a measurement and implementation handoff: evidence, target route, intended change, priority, verification, and owning task for each recommendation; record dated search observations and optional available Search Console measurements.
- [x] Implement the user-approved extension: specific MCP guide metadata, factual data/refresh answers, verified quick setup, and English `/docs/mcp` sitemap coverage with a regression check; leave homepage work to task 0003.

## Acceptance

- [x] `specdev check` reports no errors or task-specific quality warnings after scope is declared, and scoped `git diff --check` passes.
- [x] The report records the research date, exact queries used, inspected source URLs, and a comparison of relevant alternatives. Claims about their capabilities are supported by provider documentation or repository evidence.
- [x] The keyword shortlist includes intent, priority rationale, target route, and the content needed for that intent. Search volume, difficulty, and traffic forecasts are omitted unless measured with an identified source.
- [x] Each proposed Exodata claim is traceable to an existing documented capability. The report distinguishes observed facts, inference, and recommendations; it does not assert market superiority or uniqueness without evidence.
- [x] Proposed links and route targets exist, and recommendations remain limited to MCP discovery and related keywords. No website, deployment, external account, or existing audit changes are included in this research task.
- [x] The baseline records `/docs/mcp` sitemap coverage, page metadata and visible content, and connection-instruction discrepancies. Each item distinguishes repository evidence from verified deployed behavior.
- [x] The handoff defines indexing, relevant-query impressions, clicks, and CTR as follow-up measurements, with collection dates and missing-data limitations. The audit's internal score, ranking guarantees, and unsupported demand estimates are not acceptance targets.

### Implementation extension

- [x] `cargo test --locked -p exodata-web --features ssr --lib test_sitemap_static` passes with `/docs/mcp` included.
- [x] Guide links resolve and client instructions agree with current CLI help or primary documentation, without writing client configuration.
- [x] `cargo fmt --all`, scoped `git diff --check`, and `specdev check` pass without new task-specific quality warnings.

## Summary

Research exoplanet MCP discovery and implement specific guide metadata, verified setup, data provenance answers, and MCP sitemap coverage.

## Manual checks

- [ ] After the task 0003 split build, run `EXO_DATA_DIR=end2end/runtime-data cargo leptos watch --split` and open `http://127.0.0.1:3000/docs/mcp`; inspect the title/description/canonical, quick setup, supported data, limits, and examples.
- [ ] Open `http://127.0.0.1:3000/sitemap-static.xml` and confirm the canonical English `/docs/mcp` entry. Deployment and production inspection remain deferred.

## Context

- Implementation extension approved on 2026-10-06: the user requested research followed by SEO/GEO changes and approved the implementation plan. Reopened through specdev's legal approval → fix transition. Historical research boundaries and review below describe the completed research pass; this extension now owns guide specificity, setup consistency, and local sitemap coverage previously proposed for task 0012.
- [Issue #146](https://github.com/oiwn/exodata/issues/146) requests research and relevant keywords for exoplanets MCP discovery. It contains no additional comments or detailed acceptance criteria as of 2026-10-06.
- The user approved starting this research. Tasks share the existing batch PR and can land as separate commits before the batch merge; task 0001 is already implemented and awaiting that merge rather than further work.
- Deliverable: `specs/exoplanets-mcp-seo.md`, a technical research note containing dated evidence, keyword intent, route mapping, and actionable content requirements. No website implementation or deployment is included.
- Audience: English-speaking users looking to connect an AI client or agent to exoplanet data. Chinese and Japanese keyword research is excluded from this first pass unless the user expands the scope. Search Console data is optional; do not require new external account access to complete the research.

## Findings

- Existing `specs/exodata_seo_audit.md` is dated 2026-09-18 and covers broad technical SEO, routing, localization, and performance. Preserve it and independently verify any finding relevant to MCP discovery; this task does not repeat the whole audit.
- `docs/mcp.md` documents the hosted `/mcp` endpoint, Streamable HTTP, six read-only tools, schema discovery, SQL queries, and detail exports. It is the repository starting point for capability claims.
- The homepage's MCP section is rendered by `src/components/homepage_manual.rs`, with introductory prose from `docs/index.md` and localized strings. Public entry points are `/` and `/docs/mcp`; the protocol endpoint `/mcp` is a connection target, not the proposed search landing page.
- Task 0003, sourced from issue #148, covers improving homepage MCP messaging. This research may inform that task's copy and keywords, but does not implement those changes or add a formal dependency.
- Issue #159 owns broader SEO audit fixes in a separate draft task. Task 0002 recommends MCP discovery requirements; task 0003 consumes homepage messaging recommendations, while issue #159 handles technical SEO fixes after reproduction and scope review.
- Draft preparation did not establish search demand. The completed [research report](../exoplanets-mcp-seo.md) records current search observations and explicitly leaves demand measurements unknown. The issue's superiority claim is motivation, not an established result.
- Repository review found `/docs/mcp` absent from `build_static_urls` in `src/server/handlers.rs`, while `/docs`, `/docs/cli`, and `/docs/api` are included. Verify the deployed sitemap and record an actionable follow-up under issue #159; omission alone does not prove that the page is unindexed.
- `src/components/docs/registry.rs` uses the generic MCP title MCP Server. The homepage title/description and hero emphasize catalog browsing. Research should propose more specific keyword-aligned metadata and visible copy rather than merely adding meta keywords.
- `docs/mcp.md` states that `codex mcp add` only supports stdio, while `src/components/homepage_manual.rs` shows `codex mcp add ... --url`. Establish the current supported client instructions from primary documentation or CLI help before recommending consistent content.
- The audit's missing-docs-metadata claim is only partly applicable: `src/components/docs/page.rs` already emits a title, description, and canonical. `/about` redirects to `/docs`; assess missing trust or methodology content rather than assuming no overview exists.

## Verification findings

- Implementation extension: sitemap regression failed before the fix and passed after it (one test); JSON/TOML examples and relative links parsed successfully. Installed Codex/Claude help and primary Codex, Claude, OpenCode, and Crush documentation verified transport/configuration syntax. Claude project scope is now explicit; Crush's current `crushrc` form is documented alongside its supported legacy JSON. No add command or live connection was run. Split build and browser checks are handed off with task 0003.
- The task 0003 split SSR/hydration build subsequently passed on 2026-10-06 and includes these guide/metadata/sitemap changes. Both tasks await the recorded local manual checks; task 0002 is parked pending review and the shared batch merge so specdev can keep task 0003 active.
- The report records research on 2026-10-06, ten exact search queries, five primary-source comparisons, 17 keyword candidates, and a five-phrase prioritized shortlist with route and content requirements.
- NASA/IPAC AstroFetch documents a public MCP connection, so Exodata cannot claim unique hosted access. Proposed positioning is grounded in the repository's schema, read-only SQL, insight, and detail-export capabilities.
- The web research tool retrieved Exodata's homepage and MCP guide and surfaced the guide in search results. Direct requests returned 403 on all four baseline URLs; raw deployed metadata, robots headers, and sitemap contents remain unverified and are distinguished from repository findings in the report.
- Local `codex-cli 0.160.0` help supports `codex mcp add --url`; the stale stdio-only documentation statement is assigned to follow-up work. No client configuration was changed.
- Report completeness and local link checks passed: 17 candidates, five shortlist entries, no missing relative link targets. `git diff --check` passed; `specdev check` reported zero errors and no task-specific warnings.
- No Search Console access, search-volume dataset, competitor installation, website change, or background server was used. Missing deployed-head and metrics evidence is reported as a limitation, not invented or treated as zero.

## Proposed research boundary

- Start with queries such as `exoplanet MCP server`, `exoplanets MCP`, `NASA Exoplanet Archive MCP`, and `astronomy MCP server`; refine related phrases from observed results rather than inventing demand estimates.
- Inspect up to five relevant alternatives or directories. If few direct comparators exist, record the searches and distinguish indirect alternatives; do not infer that Exodata is the only service.
- Prioritize accurate discovery and connection intent. Identify the supported feature differences that help users choose or connect, and map each recommended phrase to one existing route to avoid competing page recommendations.
- Keyword phrases must inform titles, headings, descriptions, and useful visible content. [Google does not use meta keywords for ranking](https://developers.google.com/search/blog/2009/09/google-does-not-use-keywords-meta-tag).
- Assess documentation-specific structured data only as a sourced follow-up recommendation, not by copying record-page Dataset markup or promising rich results. [Google's structured-data guidelines](https://developers.google.com/search/docs/appearance/structured-data/sd-policies) do not guarantee search appearance.
- Treat `llms.txt` as an optional agent-discovery follow-up, not a prerequisite for this keyword task. Record and justify any recommendation rather than assuming a ranking benefit.
- Output research and content requirements only. Technical SEO fixes from the broad audit, directory submissions, publishing, analytics access, backlink campaigns, and website copy or metadata changes require separate authorized work.

## Review

- Implementation extension reviewed: metadata is specific to exoplanet MCP discovery, guide claims match current prepared-data and query/export contracts, and only the English setup landing page is added to the sitemap. Local focused checks passed; deployment and manual rendered-page inspection remain deferred as requested.
- Approved against the research plan and scope: the deliverable provides dated queries, primary-source comparisons, capability evidence, 17 candidates, a five-phrase shortlist, proposed metadata/headings, and a measurement/implementation handoff without website changes.
- Provider documentation is distinguished from tested runtime behavior; keyword priorities are explicit editorial judgments. The report does not invent traffic metrics, ranking guarantees, or uniqueness, and it records raw HTTP 403 limitations alongside repository and retrieved-page evidence.
- Follow-up ownership remains task 0003 for homepage messaging and task 0012 for reviewed technical SEO/documentation fixes. No new dependency or implementation scope was silently added to those tasks.
- Final local-link/completeness checks and `git diff --check` passed. `specdev check` reports zero errors and no task-specific warnings. This documentation-only task needs no Rust build or background process.

## Log

- 2026-10-05 created: exoplanets mcp seo research
- 2026-10-05 set source https://github.com/oiwn/exodata/issues/146
- 2026-10-06 scope add specs/exoplanets-mcp-seo.md — Approved research report: dated evidence, keywords, route baseline, and implementation handoff
- 2026-10-06 advance draft → ready
- 2026-10-06 scope approved: specs/exoplanets-mcp-seo.md
- 2026-10-06 advance ready → in-progress/implement
- 2026-10-06 advance in-progress/implement → in-progress/verify
- 2026-10-06 advance in-progress/verify → in-progress/review
- 2026-10-06 advance in-progress/review → approval
- 2026-10-06 advance approval → in-progress/fix (attempts 1)
- 2026-10-06 scope add docs/mcp.md — User-approved MCP guide and sitemap implementation
- 2026-10-06 scope add src/components/docs/registry.rs — User-approved MCP guide and sitemap implementation
- 2026-10-06 scope add src/server/handlers.rs — User-approved MCP guide and sitemap implementation
- 2026-10-06 scope add src/server/tests.rs — User-approved MCP guide and sitemap implementation
- 2026-10-06 advance in-progress/fix → in-progress/verify
- 2026-10-06 advance in-progress/verify → in-progress/review
- 2026-10-06 advance in-progress/review → approval
- 2026-10-06 blocked: Guide and sitemap implementation verified; awaiting local manual review and shared batch PR merge while task 0003 proceeds
