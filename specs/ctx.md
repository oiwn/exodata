# Current Task Context

## Website Content and Multilingual Layout Plan

This is planning context for a future implementation session. Do not implement
the website changes from this file until explicitly requested.

Primary direction:

1. First improve the English website content and information architecture.
2. Then add Chinese and Japanese interface localization on top of the improved
   English text.

The multilingual work should not start by translating weak or temporary English
copy. English content becomes the source text, then localized interface strings
follow.

## English Content First

### Homepage Manual

Implemented on 2026-06-28.

- Added a distinct rounded homepage manual section in
  `src/components/homepage_manual.rs`, rendered after the detailed overview
  statistics and reachable from the hero subtitle link.
- Source prose now lives in `docs/index.md` and is rendered with the existing
  docs Markdown renderer.
- The section includes stable catalog example links, hosted MCP setup snippets
  for Codex, Claude Code, OpenCode, and MCP Inspector, copy buttons, and a
  compact CLI/MCP interaction card.
- `docs/mcp.md` now documents only the hosted MCP URL in the connection summary.

### Docs and Public Copy Cleanup

Before translation, tighten the English source copy:

- Make homepage/manual text concise and stable.
- Make docs index copy suitable as the English source for translation.
- Keep API/MCP/CLI examples exact and technical.
- Separate prose that should be translated from command examples, URLs, JSON
  keys, SQL, and dataset field names that should remain unchanged.

### Blog and Changelog Ideas

Do not add a blog route in the first content pass. Keep these as future content
ideas:

- story posts about why tables are not enough for exoplanet catalogs
- agent/MCP usage stories
- small dataset stories based on insights pages
- human-readable changelog summaries for major project milestones
- technical notes about VOTable to Parquet to web/API/CLI flow

`CHANGELOG.md` should remain technical. A future public notes page can summarize
selected changes in a more narrative format.

### AI Test-Generation Research Idea

Keep this as a research spike, not production automation:

- Explore using `rig.rs` with a DeepSeek Flash model to generate small candidate
  tests for distinct stellar-system and planet-system features.
- Generated tests must be reviewed manually before committing.
- Do not wire model calls into CI in the first experiment.
- Do not infer behavior from exact fixture numbers when those numbers are only a
  sampled snapshot of the live dataset.

## Multilingual Interface Plan

Goal: add Chinese and Japanese interface support after the English content pass,
without turning the first iteration into a full content-management platform.

### Target Languages

- Default: English (`en`)
- Add: Simplified Chinese (`zh-CN`)
- Add: Japanese (`ja`)

UI labels:

| Locale | Switcher label | Native label |
|---|---|---|
| `en` | EN | English |
| `zh-CN` | 中文 | 简体中文 |
| `ja` | 日本語 | 日本語 |

Traditional Chinese is out of scope unless explicitly added later.

### URL Layout

Use prefixed locale routes for localized website pages:

- `/` remains the canonical English homepage.
- `/zh-CN` renders the Chinese homepage.
- `/ja` renders the Japanese homepage.
- English routes remain valid without prefixes:
  - `/stellarhosts`
  - `/exoplanets`
  - `/insights`
  - `/docs`
  - detail routes
- Localized interface routes use the locale prefix:
  - `/zh-CN/stellarhosts`
  - `/ja/stellarhosts`
  - `/zh-CN/exoplanets`
  - `/ja/exoplanets`
  - `/zh-CN/insights`
  - `/ja/insights`

Do not move non-website utility surfaces under locale prefixes:

- keep `/rest/...`
- keep `/mcp`
- keep `/swagger-ui`
- keep `/sitemap-index.xml` and child sitemap routes
- keep current JSON/CSV export URL behavior

Rationale: prefixed routes are clear for users and SEO, while unprefixed English
routes preserve existing URLs.

### Locale Detection and Rendering

- Derive the active locale from the first path segment.
- If the first path segment is `zh-CN` or `ja`, render interface strings in that
  locale.
- Otherwise render English.
- Data rows, entity names, API paths, SQL, JSON keys, CSV headers, and NASA field
  keys remain unchanged.
- Detail route identifiers remain original catalog names and are not translated.

The routing layer should avoid duplicating page implementations. Prefer a shared
locale context/provider that all components can read.

### Language Picker Component

Plan a dedicated language picker component rather than scattering language links
through the navbar.

Behavior:

- Render compact options: `EN`, `中文`, `日本語`.
- Show the active locale visually.
- Map the current page to the same page in the selected locale where possible:
  - `/exoplanets` -> `/ja/exoplanets`
  - `/ja/exoplanets` -> `/zh-CN/exoplanets`
  - `/stellarhosts/TRAPPIST-1` -> `/zh-CN/stellarhosts/TRAPPIST-1`
- Preserve query strings for table pages when switching languages.
- Do not prefix external links or utility routes.

Placement:

- Desktop: navbar, compact control near the right side.
- Mobile: inside the mobile menu, preferably as a segmented row near the bottom.

Accessibility:

- Use a clear accessible label such as `Language`.
- Each option should expose the native language name, not only the compact label.
- Keyboard focus and current-state styling should be visible.

### Translation Technology

Use a real i18n crate instead of hand-written string tables if it works cleanly
with the current Leptos version, SSR, and hydration.

Preferred candidate for the future implementation spike:

- `leptos_i18n`

Why:

- It is designed for Leptos applications.
- It supports compile-time translation resources and component-level usage.
- It is a better long-term fit than ad hoc match statements once interface copy
  spans navbar, tables, details, insights, docs chrome, metadata, and aria labels.

Spike requirements before committing to it:

- Confirm compatibility with the repository's Leptos version.
- Confirm SSR and hydrate builds both work.
- Confirm translations can be selected from URL-derived locale state.
- Confirm metadata/head strings can use localized values.
- Confirm the generated code does not make simple UI text awkward to maintain.

Fallback if `leptos_i18n` does not fit:

- Use a small internal translation module with semantic keys for v1.
- Keep the API shaped so it can be replaced by an i18n crate later.

Do not translate directly by using English strings as lookup keys. Use semantic
keys or generated typed accessors so English copy can be edited without breaking
translations.

### Translation Surface

Translate interface chrome and project-authored prose first:

- navbar labels, mobile menu labels, aria labels
- language picker labels
- homepage hero/manual text and stat labels
- loading, error, empty, and not-found states
- table page headings and pagination text
- column selector controls
- insight page headings, cards, empty states, and explanatory text
- detail page section headings, provenance/download controls, and summary labels
- docs registry titles/descriptions and short localized docs summaries

Do not translate source data values in v1:

- planet names, stellar host names, system names
- discovery methods, facilities, references, aliases, and record values
- raw column keys such as `pl_name`, `hostname`, `st_teff`, and `sy_dist`
- SQL examples, API paths, CLI commands, JSON keys, CSV headers

Scientific units stay unchanged in v1: `K`, `pc`, `R⊕`, `M⊕`, `R☉`.

### Docs Content

Use separate Markdown files only for prose-heavy translated docs.

Suggested future layout:

```text
docs/
├── about.md
├── api.md
├── cli.md
├── mcp.md
└── i18n/
    ├── zh-CN/
    │   ├── about.md
    │   ├── api.md
    │   ├── cli.md
    │   └── mcp.md
    └── ja/
        ├── about.md
        ├── api.md
        ├── cli.md
        └── mcp.md
```

For the first localized docs pass, short localized summaries are enough. They
can link to full English technical docs until full translations are ready.

### SEO and Metadata

- Set `<html lang>` from the active locale.
- Canonical URLs should point to the active localized page.
- Add `hreflang` alternates for English, Chinese, and Japanese equivalents.
- Include localized static routes in the sitemap after route behavior is stable.
- Do not add localized copies of every detail route to the sitemap in v1.
- Keep existing English detail routes as the primary entity URLs.

### Layout Requirements

- Use compact labels in the language picker to avoid crowding the navbar.
- Avoid fixed-width text containers for translated copy.
- Check Chinese and Japanese line wrapping in:
  - homepage cards/manual section
  - table controls and pagination
  - column selector
  - insight cards
  - detail-page download buttons
- Preserve readable density. The localized interface should not become a separate
  visual design.

### Suggested Rollout

1. Polish English homepage/manual/docs source copy.
2. Add locale route parsing, locale context, language picker, and localized href
   generation while still rendering English strings.
3. Integrate the chosen i18n approach and translate global navigation plus common
   states.
4. Translate homepage manual, table controls, insight chrome, and detail chrome.
5. Add localized docs summaries.
6. Add localized metadata, `hreflang`, and sitemap entries.

### Verification

Unit tests:

- locale parsing from path
- localized href generation with query-string preservation
- utility routes remain unprefixed
- canonical and alternate URL generation
- docs link rewriting with locale prefixes

SSR/manual checks:

- `/`, `/zh-CN`, `/ja`
- `/zh-CN/stellarhosts`, `/ja/exoplanets`
- localized detail route switching
- table query state survives language switching
- `/rest`, `/mcp`, `/swagger-ui`, and export URLs remain unprefixed

Browser checks:

- desktop navbar does not overflow
- mobile language picker is usable
- CN/JP text does not overlap in cards, table controls, pagination, and detail
  action buttons
