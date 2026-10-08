# Exoplanets MCP Discovery Research

Research date: 2026-10-06. Implements the research deliverable for [task 0002](tasks/0002-exoplanets-mcp-seo-research.md) and [issue #146](https://github.com/oiwn/exodata/issues/146). Audience: English-speaking users connecting AI clients to exoplanet records. This report proposes content requirements and technical follow-ups; it does not implement or publish them.

## Outcome and evidence limits

- Recommend `/docs/mcp` as the primary landing page for MCP server, query, and setup searches; use `/` for broader exoplanet-data discovery and a clear link to the connection guide. Keep `https://exodata.space/mcp` as the protocol endpoint, not a search landing page.
- Position Exodata through its documented hosted connection, schema inspection, read-only SQL over two catalog tables, curated insights, and JSON/CSV detail exports. Do not claim that it is the only or best exoplanet MCP service.
- [NASA's October 1 announcement](https://exoplanet.ipac.caltech.edu/docs/exonews_archive.html) and [AstroFetch's agent guide](https://astrofetch.ipac.caltech.edu/agents) document a public MCP service. Other inspected repositories also support exoplanet queries. These are documented alternatives, not benchmarked services.
- Keyword priorities below are editorial judgments based on product fit and observed result types. No search volumes, keyword difficulty scores, traffic forecasts, rank positions, or Search Console statistics were obtained. No authenticated accounts, installations, client configuration writes, or background servers were needed.

## Search observations

The research search tool ran these exact queries on the research date. Its combined results are not a controlled, location-specific Google SERP, and result order is not a ranking measurement.

- `exoplanet MCP server`
- `exoplanets MCP`
- `NASA Exoplanet Archive MCP`
- `astronomy MCP server`
- `"exoplanet" "MCP" server github`
- `"exoplanets MCP"`
- `"NASA" "MCP" "exoplanet" github`
- `"exoplanet MCP server" "Exodata"`
- `"NASA Exoplanet Archive MCP" SQL`
- `"exoplanet MCP" "Claude Code"`

Observed result types included provider repositories, NASA documentation, MCP directories and packages, Exodata's MCP docs, and unrelated astronomy publications. Broad MCP phrases also surfaced detector hardware and magnetic-star terminology; explicitly naming Model Context Protocol and MCP server is a content-disambiguation recommendation, not a measured ranking benefit.

[Exodata's MCP guide](https://exodata.space/docs/mcp) appeared in the targeted search results. This establishes visibility in the research search tool's indexed results; it does not establish Google indexing status, query impressions, or a stable position.

## Inspected alternatives

Five examples were inspected through provider documentation or repositories. Availability and feature descriptions are documentation evidence; no competitor endpoint was benchmarked or connected to a client. The sample is not exhaustive.

- **NASA/IPAC AstroFetch — direct hosted alternative.** [Agent guide](https://astrofetch.ipac.caltech.edu/agents) documents Streamable HTTP at `https://astrofetch.ipac.caltech.edu/mcp`, without authentication for that connection, and access to multiple archive and literature services. Its examples include natural-language requests and ADQL-backed results. Exodata should describe its explicit schema/SQL/export workflow without claiming unique hosted or keyless access.
- **saikrmet/nasa-exoplanet-mcp — direct archive wrapper.** [Repository](https://github.com/saikrmet/nasa-exoplanet-mcp) documents archive searches, object details, name resolution, table/column discovery, and ADQL queries, with local stdio and self-hosted HTTP modes. The inspected README provides a host placeholder rather than an advertised public endpoint. Do not portray schema discovery or flexible queries as unique to Exodata.
- **ProgramComputer/NASA-MCP-server — broader NASA wrapper.** [Repository](https://github.com/ProgramComputer/NASA-MCP-server) documents Exoplanet Archive access among many NASA tools, with stdio and optional self-hosted Streamable HTTP. Its current README distinguishes key requirements by upstream service. Exodata's narrower catalog focus is a supported positioning difference; it is not proof of better coverage or accuracy.
- **cyanheads/astronomy-mcp-server — adjacent astronomy service.** [Repository](https://github.com/cyanheads/astronomy-mcp-server) advertises a public HTTP endpoint and offline sky-position, rise/set, phase, and event calculations. Its main intent is observational planning rather than reference-record SQL. Broad astronomy MCP wording reaches a different audience and is a lower-priority phrase for Exodata.
- **Wicked-Sick-Ltd/solar-system-db — adjacent catalog ecosystem.** [Repository](https://github.com/Wicked-Sick-Ltd/solar-system-db) documents an exoplanet/host extension sourced from PSCompPars, REST routes, and equivalent MCP tools. The inspected README establishes catalog capabilities but is not an end-to-end MCP deployment test. Describe Exodata's PS reference-record model accurately rather than assuming all catalogs have equivalent row semantics.

## Current discovery baseline

Evidence is separated by retrieval method. The research web tool retrieved the deployed [homepage](https://exodata.space/) and [MCP guide](https://exodata.space/docs/mcp) as page text. Direct Python HTTP requests to `/`, `/docs/mcp`, `/robots.txt`, and `/sitemap-static.xml` all returned 403; web-tool fetches of robots and the static sitemap also failed. These responses do not establish that ordinary browser visitors receive 403, or diagnose a particular edge rule. Raw deployed head tags, response robots headers, and sitemap contents remain unverified.

- **Homepage title and headings — deployed text plus repository.** The web tool reported Exoplanets Catalog | Exodata and the heading Exoplanet Archive. Its content emphasized catalog statistics. Repository `locales/en.json` supplies that general title and a description focused on browsing/searchable tables rather than the MCP connection. MCP introduction and setup are present below the statistics, supported by `src/components/homepage_manual.rs` and `docs/index.md`.
- **MCP guide title and headings — deployed text plus repository.** The web tool reported MCP Server | Exodata and an MCP Server heading, followed by tools, limits, schema-first query examples, exports, and client setup. `src/components/docs/registry.rs` supplies a description about connecting coding agents, schema, and SQL, but its title omits exoplanet and NASA context.
- **Descriptions and canonicals — repository only.** `src/components/overview.rs` emits the homepage description and locale-aware canonical. `src/components/docs/page.rs` emits the registry description and canonical for `/docs/mcp`. Expected English targets are `https://exodata.space/` and `https://exodata.space/docs/mcp`; confirm raw deployed HTML when direct inspection is possible. These tags exist in code, so adding missing tags is not the identified requirement.
- **Internal links — deployed guide plus repository.** Retrieved MCP text links its SQL interface to `/rest/query` and explains the connection endpoint. Homepage source links to `/docs/mcp` and exposes setup information; the guide contains no-installation HTTP setup examples. Preserve a normal crawlable anchor from the homepage to the guide, and verify the rendered link when implementing new copy.
- **Robots directives — repository only.** `public/robots.txt` allows all crawlers and names `https://exodata.space/sitemap-index.xml`. No page-specific noindex directive is introduced by the inspected homepage or docs components. Response-level directives and deployed robots content were not verified.
- **Static sitemap — repository only.** `build_static_urls` in `src/server/handlers.rs` includes `/docs`, `/docs/cli`, and `/docs/api`, but omits `/docs/mcp`. Add a reproduced sitemap follow-up under task 0012; do not infer that omission prevents indexing, particularly since the guide appeared in the research search results.
- **Connection consistency — repository and local CLI.** `docs/mcp.md` says Codex's add command is stdio-only, while homepage code supplies `codex mcp add exodata --url https://exodata.space/mcp`. Local `codex-cli 0.160.0` help explicitly supports `--url` for Streamable HTTP; [AstroFetch's primary guide](https://astrofetch.ipac.caltech.edu/agents) also documents that command form. Correct the stale docs statement in follow-up work. The help command was inspected; no MCP configuration was added and no live connection was tested.

## Supported positioning

Repository evidence: `docs/mcp.md`, `src/server/mcp.rs`, and `specs/data-management.md`.

- Exodata documents a hosted Streamable HTTP connection at `https://exodata.space/mcp` and six tools: `health`, `list_insights`, `run_insight`, `describe_catalog`, `query_catalog`, and `download_detail`.
- Agents can inspect columns, descriptions, units, and types, then run one read-only SQL SELECT against `stellarhosts` and `exoplanets`, including supported joins and aggregations. MCP query responses default to 100 rows and cap at 1,000, with a 30-second query timeout.
- Detail exports return JSON or CSV for a named host or planet. Do not describe this as unlimited bulk query export; the docs distinguish query rows from detail downloads.
- The catalog is loaded from prepared NASA exports. The exoplanets dataset uses PS reference records, not one composite row per planet. Avoid implying live upstream requests, guaranteed daily refresh, official NASA operation, full archive-table coverage, or scientifically validated AI answers.
- A truthful concise description is: Exodata provides hosted MCP access to NASA Exoplanet Archive records, with schema discovery, read-only SQL, curated insights, and JSON/CSV detail exports. These capabilities can be described together without asserting competitor absence or superiority.

## Keyword candidates

These 17 phrases are candidates, not measured search demand. Each has one primary route. Variants share a content cluster rather than requiring new pages.

Connection/setup intent:

- `exoplanet MCP server` — `/docs/mcp`; primary server-discovery phrase.
- `exoplanets MCP` — `/docs/mcp`; plural variant, disambiguate the protocol explicitly.
- `hosted exoplanet MCP server` — `/docs/mcp`; remote connection without a local server install.
- `NASA Exoplanet Archive MCP` — `/docs/mcp`; archive-data intent; make third-party operation clear.
- `Claude Code exoplanet MCP` — `/docs/mcp`; client connection instructions.
- `Codex exoplanet MCP` — `/docs/mcp`; client connection instructions.

Catalog-query intent:

- `read-only exoplanet SQL MCP` — `/docs/mcp`; schema-first SELECT examples and limits.
- `exoplanet SQL MCP` — `/docs/mcp`; shorter query variant.
- `exoplanet schema discovery MCP` — `/docs/mcp`; explain describe_catalog output.
- `query NASA exoplanet data with AI` — `/docs/mcp`; demonstrate results grounded in returned records.
- `export exoplanet data MCP` — `/docs/mcp`; distinguish detail exports from bounded query responses.

Data-access/discovery intent:

- `exoplanet data for AI agents` — `/`; product discovery, then link to connection instructions.
- `NASA exoplanet data AI` — `/`; broad source-and-agent phrase, avoid implying official NASA operation.
- `stellar host data MCP` — `/docs/mcp`; explain the second table and supported host queries.
- `exoplanet catalog Model Context Protocol` — `/docs/mcp`; spell out MCP for ambiguous searches.
- `astronomy MCP server` — `/docs/mcp`; secondary only, because many observed results serve different astronomy workflows.
- `NASA MCP server` — `/docs/mcp`; secondary only, because broad NASA wrappers cover many unrelated services.

## Prioritized shortlist

Priority is based on direct intent and documented fit, not volume or difficulty. Lower-case/plural variants belong to the same page and need not be repeated unnaturally.

1. **P1: exoplanet MCP server — `/docs/mcp`.** The sampled results contain dedicated MCP providers and this is the exact product category. Required content: explicit exoplanet MCP title/H1, hosted URL, transport, tool list, and a minimal working schema/query example.
2. **P1: NASA Exoplanet Archive MCP — `/docs/mcp`.** Archive documentation and archive wrappers appeared in targeted results. Required content: source attribution, third-party-service identity, supported tables, PS row semantics, and a link to the archive rather than an official-service implication.
3. **P1: read-only exoplanet SQL MCP — `/docs/mcp`.** This is a product-fit hypothesis drawn from Exodata's schema/SELECT workflow and inspected ADQL/SQL alternatives, not a proven popular phrase. Required content: schema-first flow, one valid query, supported operations, limits, exports, and the SQL versus upstream ADQL distinction.
4. **P2: Claude Code exoplanet MCP — `/docs/mcp`.** Provider setup guides establish a concrete connection intent. Required content: verified Claude Code command, adjacent Codex HTTP instructions, configuration scope, and a health check. Do not assume Claude demand exceeds Codex demand without measurements.
5. **P2: exoplanet data for AI agents — `/`.** The homepage already combines browsing and an MCP introduction, while archive services explicitly address AI-assistant use. Required content: a clear MCP value statement beside browsing, a descriptive guide link, and a short explanation of what agents can query. This broader phrase is an editorial discovery hypothesis.

## Proposed page content

Drafts for follow-up review; no copy or metadata is changed by this research task.

- **Homepage title:** Exoplanet Data & MCP Server | Exodata.
- **Homepage description:** Browse NASA Exoplanet Archive records or connect your AI agent to Exodata's hosted MCP server for read-only SQL, schema discovery, and JSON/CSV detail exports.
- **Homepage headline:** Explore exoplanets. Connect your AI agent with MCP.
- **Homepage content:** retain catalog browsing and statistics; add an explicit hosted MCP introduction, three grounded capabilities, source attribution, and a clear connection-guide link. Task 0003 owns this messaging work.
- **MCP guide title:** Exoplanet MCP Server: NASA Data, SQL & Setup | Exodata.
- **MCP guide description:** Connect Claude Code, Codex, or another HTTP MCP client to query NASA Exoplanet Archive records. Inspect schemas, run read-only SQL, and export JSON/CSV details.
- **MCP guide headline:** Connect to Exodata's exoplanet MCP server.
- **MCP guide sections:** supported data and reference rows; endpoint and transport; verified client setup; six tools; schema-first query and export examples; limits and refresh behavior; related API and catalog links. Correct connection inconsistencies before promoting setup phrases.
- Keyword phrases belong in useful visible content, titles, and headings. [Google ignores meta keywords for ranking](https://developers.google.com/search/blog/2009/09/google-does-not-use-keywords-meta-tag). Do not treat the registry's keywords field as the implementation outcome.
- Consider documentation-appropriate structured data only after the content is correct; do not copy record-page Dataset markup onto the setup guide or promise rich results. [Google's structured-data policy](https://developers.google.com/search/docs/appearance/structured-data/sd-policies) requires accurate content and does not guarantee search appearance.

## Implementation handoff

Implementation update, 2026-10-06: the user approved implementing MCP guide specificity, setup consistency, and local English sitemap coverage under task 0002, plus a brief localized homepage description and setup link under task 0003. The recommendations below preserve the original research handoff; task 0012 no longer owns those selected local fixes. Deployment, production crawler inspection, analytics access, and additional structured data remain deferred until the current task batch is implemented.

Proposed follow-ups need scope approval under their owning tasks. No new tasks, formal dependencies, website changes, directory submissions, or deployments are created by this report.

- **P1, task 0003: homepage MCP messaging.** Evidence: generic deployed title/heading and MCP material below statistics. Target: `/`. Change: adapt the proposed title, description, headline, and link using existing localized content surfaces. Verify rendered metadata, MCP explanation, guide navigation, locale behavior, and retained browsing functions.
- **P1, task 0012: MCP sitemap discovery.** Evidence: repository omission; raw deployed sitemap not verified. Target: `/sitemap-static.xml`. Change: reproduce the deployed omission, add `/docs/mcp` if appropriate, and guard the static route list. Verify sitemap content and the corresponding canonical landing URL. Omission is a discovery gap, not proof of exclusion from search.
- **P1, task 0012: MCP guide specificity and setup consistency.** Evidence: generic retrieved title, registry metadata, and the Codex docs/help discrepancy. Target: `/docs/mcp`. Change: review the proposed metadata/heading and update stale client instructions. Verify rendered content, CLI help for commands, endpoint spelling, and schema/query examples; do not run add commands against user configuration during documentation checks.
- **P2, task 0012: guide structured-data assessment.** Evidence: docs render metadata but no documentation-specific JSON-LD component was found in the inspected docs page. Target: `/docs/mcp`. Change: justify a documentation-appropriate schema and its required supported fields before implementation. Verify emitted data matches visible text; no ranking guarantee is an acceptance criterion.
- **P2, user-managed measurement: landing-page/query baseline.** Evidence: Exodata appeared in this research tool, but Google indexing and performance data are unavailable. Target: `/` and `/docs/mcp`. Change: collect the metrics below using existing authorized access if available. This is not a prerequisite for completing research.
- `llms.txt`, broad detail-page URL migration, security headers, payload optimization, planet prose, localization expansion, and backlink campaigns remain separate audit work. Do not automatically disallow query URLs: [Google distinguishes crawling from indexing](https://developers.google.com/search/docs/crawling-indexing/robots/intro).

## Measurement and validation

- **Before publishing follow-ups:** record collection date, deployed version, locale, landing URLs, page titles/headings, canonical targets, sitemap presence, and any indexing status available through the owner's Search Console. This report's search observations are the dated discovery baseline; missing metrics are unknown, not zero.
- **After publishing:** compare equal 28-day windows for the same English landing URLs and exoplanet-MCP query cluster, annotate the deployment date and content changes, and retain exact query variants. Record impressions, clicks, CTR, and Search Console average position when available; these values are not inferred from this search tool.
- **Interpretation:** CTR is clicks divided by impressions; record unavailable when impressions are zero. Treat ranking and click changes as observations, not proof that one copy edit caused them. Do not use the audit's 58/100 internal score as the target.
- **Research verification:** inspect primary provider links, local capability definitions, route declarations, current Codex help, report completeness, `specdev check`, and `git diff --check`. A Rust build or new background app is unnecessary for this documentation-only deliverable.
- **Remaining limitations:** competitor availability was not exercised, exact raw deployed metadata/robots/sitemap inspection was blocked, refresh frequency is not measured, and no Search Console or keyword-volume dataset was accessed. None of these limitations establishes poor service quality or lack of indexing.
