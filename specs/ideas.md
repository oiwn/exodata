# Ideas

Uncommitted possibilities and remaining gaps. Current behavior belongs in
technical specs; committed work belongs in [roadmap.md](roadmap.md).

## Catalog and Table UX

- [ ] Extend JSON/CSV access beyond the already implemented detail-page suffix
  exports if a concrete general-table API use case requires it.
- [ ] Add a table legend for error/limit indicators and consider collapsible
  companion values. Companion grouping and exclusion from column selection
  already exist.
- [ ] Add a dedicated REST detail-by-name endpoint if the existing detail
  exports and MCP download tool do not meet client needs.
- [ ] Evaluate cache cleanup/invalidation needs beyond the current bounded
  startup-data caches.
- [ ] Fix the column selector's "drag to reorder" label or implement actual
  dragging; current controls move columns up/down.
- [ ] Use stable column identity for selected-list keys rather than including
  the current index.
- [ ] Add wide-table reference links, column presets, and copy/share controls
  where the existing detail provenance/export UI is insufficient (#58).
- [ ] Consider a strict filter mode alongside current substring search (#45).
- [ ] Extend row-to-JSON dtype support where required: unsupported types
  currently become null. Evaluate conversion efficiency with representative data.
- [ ] Align startup prewarm query columns with the explicit first-page UI fetch
  if measurements confirm redundant cache misses (#27).

## Routing and Documentation

- [ ] Preserve locale prefixes throughout table pagination, filters, entity
  links, and detail back links; reuse existing locale helpers.
- [ ] Render missing detail entities through the branded 404 treatment (#69).
- [ ] Add useful facts/insight navigation to the 404 page (#75).
- [ ] Decide whether `/docs/mcp` and `/about` belong in the static sitemap.
  Keep localized sitemap additions subject to [localization.md](localization.md).
- [ ] Add technical documentation pages for architecture, tables, and tooling
  only when there is a public audience. Existing docs compile Markdown from
  `docs/`; publishing specs and moving rendering to a server-only path remain
  separate choices.
- [ ] Add a navigation progress indicator (#108).
- [ ] Evaluate API/CLI tabs on the homepage (#82).
- [ ] Consider changing the manual fragment to `#manual` (#119), accounting for
  existing links and stable cross-locale fragments before changing it.
- [ ] Consider short URLs (#104), arXiv integration (#100), and `llms.txt`
  (#98) as separate features with explicit acceptance criteria.
- [ ] Finish page-specific translations and metadata as described in
  [localization.md](localization.md).

## Detail Page Follow-ups

- [ ] Decide whether planet hero and comparison should consume the canonical
  payload rather than separately deriving values from records. Include mass
  fallback consistency and stable/categorical selections in that decision.
- [ ] Consider separate `identity`, `visual`, and `provenance` payload
  fields only when a consumer needs them: hostname identity, deterministic
  visual inputs, and reference/measurement summaries. Reuse shared summary
  types; do not add parallel formatting models speculatively.
- [ ] Evaluate curated visual style mappings versus the current deterministic
  measured-field classification.
- [ ] Consider mass/density/gravity comparison modes beyond the existing
  radius-only Earth/Jupiter comparison.
- [ ] Add binary-system visualization to host details if specified (#77);
  the binary-systems insight already exists.
- [ ] Consider explicit REST/API links on detail pages beyond JSON/CSV downloads
  (#59).
- [ ] Continue semantic-class cleanup for remaining utility-heavy components,
  including the footer (#129), while preserving their design.

## Data and Insight Extensions

- [ ] Consider source diffs or download versioning only as a separate feature.
  They are excluded from the current description-generation task and data workflow.
- [ ] Consider additional VOTable FIELD attributes (such as UCD, ID, arraysize)
  or metadata-completeness validation when a consumer requires them.
- [ ] Consider static result snapshots for fixed insight rankings if repeated
  query execution becomes unnecessary.
- [ ] Evaluate new insights: recent planets; hottest/coldest planets;
  coolest, largest/smallest, or most massive hosts; hosts with the largest
  planet; compact systems; equal planet sizes; hottest planets around cool
  hosts; nearby multi-planet systems; densest/least-dense planets; and oldest
  hosts or hosts with planets. Check the current registry before specifying a
  new ranking; existing smallest/largest planets, nearest/hottest hosts,
  crowded/binary systems, equal star-planet pairs, and radius ratios are implemented.

## Other Unspecified Enhancements

These remain exploratory and need a concrete use case before implementation:

- [ ] Dark/light themes, bookmarks, broader comparison tools, and offline/PWA
  support with client caching or service workers.
- [ ] Virtual scrolling, image lazy loading, and prefetching where performance
  measurements justify them; route-based WASM splitting already exists.
- [ ] Keyboard shortcuts, further screen-reader improvements, high-contrast
  presentation, and reduced-motion support.
- [ ] Alternative database storage, external caches, GraphQL or WebSocket
  interfaces, authentication, rate limiting, and explicit CORS or application
  compression requirements beyond the current deployment configuration.

## Project Development Harness Ideas

The implemented [checks skill](../.agents/skills/exodata-checks/SKILL.md) and
[data skill](../.agents/skills/exodata-data/SKILL.md) cover verification and local
data workflows. Remaining candidates:

- [ ] **`exodata-web`: frontend development guidance.** Route work through the
  shared table/metadata helpers, semantic feature styles, lazy routes, and
  localization contracts. Include a docs/route checklist spanning registry,
  Markdown link rewriting, page metadata, and sitemap eligibility. Reuse the
  checks skill for execution instead of duplicating browser setup.
- [ ] **Backend/API maintenance skill**, if repeated work justifies it. Explain
  tracing changes through shared data functions, REST/OpenAPI, Leptos, MCP,
  and CLI backends; affected-interface verification already belongs to checks.
- [ ] Consider a small local path/link checker if repeated maintenance warrants
  an executable helper. Report stale references without automatically changing
  contracts or rewriting specs.

Keep only project development workflows in tracked `.agents/skills/`.
The public catalog-query skill remains maintained with the CLI at
[crates/exo-cli/skills/exodata.md](../crates/exo-cli/skills/exodata.md);
its generated installation and external `specdev` installation stay ignored.
Product narrative guides and personal terminal preferences belong elsewhere.
