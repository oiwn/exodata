# Web Frontend

The website uses Leptos SSR followed by WASM hydration. Route components,
shared UI state, and semantic feature styles are maintained separately from
server data processing.

## Shell, Routes, and Localization

[app.rs](../src/app.rs) provides metadata and locale contexts, the navbar,
one app-wide `<main>`, route fallback, and footer. Its shell embeds the shared
metadata JSON and hydration scripts. A pre-hydration overlay blocks interaction;
[lib.rs](../src/lib.rs) removes it after hydration.

Current page routes are listed in [overview.md](overview.md#website). They use
lazy components and `SsrMode::Async`. English routes are unprefixed;
Simplified Chinese and Japanese aliases use `/zh-CN` and `/ja`. Route
declarations currently repeat across locales while sharing page implementations.

The URL selects the locale. The language switcher preserves path, query, and
fragment. REST, MCP, Swagger, sitemap, and JSON/CSV exports remain unprefixed.
Global navigation and homepage content are translated; remaining page-specific
strings and metadata are tracked in [localization.md](localization.md).
Some table and detail links still use unprefixed paths; full locale retention
during page navigation remains unfinished.

## Module Boundaries

- `components.rs` and `components/` expose lazy page features and shared UI.
- `table.rs` and `table/` own query state, navigation, pagination, column
  grouping, and rendering shared by the two catalog pages.
- `metadata.rs` owns the app-wide metadata store; `locale.rs` owns URL helpers.
- `metadata_helpers.rs` and `structured_data.rs` support page metadata,
  canonical URLs, encoded entity paths, and structured data.
- `server/functions.rs` defines shared serializable payloads. Its server
  function bodies execute on the server; payload types are not duplicated
  independently for SSR and hydration.

## Catalog Tables

`/stellarhosts` and `/exoplanets` have separate lazy page resources and
dataset-specific defaults. They share `TableQueryState` transitions and
`CatalogTableResult` rendering.

URL parameters are `page`, `sort`, `order`, `columns`, and `filter`.
Page defaults to 1 and is omitted from canonical query output at page 1.
Sorting cycles ascending, descending, then cleared. Sorting, selected-column
changes, and filter commits reset to page 1; pagination preserves other state.
Removing a sort column clears the sort. Companion error/limit columns are
excluded from the parsed display-column selection.

Invalid page values and pages beyond the filtered result range render the
branded 404; zero matching rows still allow page 1. These website rules differ
from backend page normalization; see [web-backend.md](web-backend.md#tables-and-schema).

The shared metadata store supplies header tooltips and the column selector,
including after client navigation from the homepage. See
[column-metadata.md](column-metadata.md). Column order controls currently use
up/down buttons; drag-and-drop is not implemented.

## Overview, Details, Insights, and Docs

`OverviewPage` fetches precomputed `DataStats` using `get_stats`. It renders
summary cards and paired distribution blocks:

1. Planet classifications / orbital periods
2. Planet mass distribution / stellar classes
3. Discovery methods / discovery years
4. Planet temperature bands / detection sources

Planet distributions count distinct planets using a canonical value per planet.
The mass block uses median `pl_bmasse` values and fixed Earth-mass bands. The
stellar-class block counts distinct hosts by the normalized leading letter of a
canonical `st_spectype` value and displays the five most common classes. Its
standard O/B/A/F/G/K/M row labels use localized conventional names such as
yellow dwarf, orange dwarf, and red dwarf. These are class-level shorthand:
because aggregation ignores the luminosity-class suffix, a row can also include
giants of the same spectral class. An unrecognized leading letter remains
unchanged. Planet-size categories, orbital-period units, temperature bands, and
known discovery methods are also localized in the presentation layer; numeric
mass/year labels and proper facility names remain language-neutral. Unknown
data-provided labels remain unchanged. All blocks use `StatSection`, which
renders each category's count and percentage of the displayed distribution.

Host details render hero/star visual, canonical summary, comparison, related
planet cards, and provenance. Host and related-planet data use separate
resources. Their contracts are in
[backend details](web-backend.md#detail-contracts).

Planet details render hero/visual, canonical summary, comparison, and provenance
with JSON/CSV exports. See [exoplanet-detail.md](exoplanet-detail.md).
Both detail pages currently render feature-specific resource errors rather
than routing all missing entities through the branded 404.

Insight page components use the frontend registry, which is tested against
shared insight metadata. Server execution uses the core insight registry.

Docs pages select compiled `docs/` Markdown through
[registry.rs](../src/components/docs/registry.rs). Rendering in
[render.rs](../src/components/docs/render.rs) rewrites known Markdown document
links to website routes. The renderer is called by the page component, not by
a dedicated server function. Unknown documentation slugs have a docs-specific
not-found view. Specs are not published automatically.

## Styling with Tailwind CSS

Tailwind CSS v4 uses `style/tailwind.css`; no JavaScript Tailwind config is
required. Semantic feature styles live in `style/components/*.css` and are
imported by that entrypoint.

- Move large or repeated page/section utility stacks into semantic classes.
- Use `@apply` for utilities and ordinary CSS for custom styling.
- Keep small one-off layout and text utilities inline when useful.
- Classes are global: use feature-specific names such as `.host-hero`,
  `.host-hero__title`, `.host-provenance`, and `.planet-comparison`.
- Preserve the existing visual design during cleanup and limit changes to
  the component being worked on.

Existing examples are
[stellarhost-detail.css](../style/components/stellarhost-detail.css) and
[exoplanet-detail.css](../style/components/exoplanet-detail.css).

## Build and Verification

Cargo Leptos builds the server with `ssr` and the client library with
`hydrate`, disabling default features for both configured targets.
Lazy routes require `--split`. The generated site is in `target/site`;
deployment copies the entire site, including split WASM chunks.

See [testing.md](testing.md) for coverage and the
[checks skill](../.agents/skills/exodata-checks/SKILL.md) for build, browser,
and focused verification commands. Future frontend work remains in
[ideas.md](ideas.md) and [roadmap.md](roadmap.md).
