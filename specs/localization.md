# Localization

## Localization Status

The English homepage content pass and the first localization pass are complete.
The implementation uses `leptos_i18n` with semantic keys and compile-time JSON
resources for:

- English (`en`) — default, unprefixed routes
- Simplified Chinese (`zh-CN`) — `/zh-CN/...`
- Japanese (`ja`) — `/ja/...`

Traditional Chinese is out of scope unless explicitly requested.

## Completed

- Added locale-prefixed aliases for website routes without duplicating page
  implementations.
- Added the desktop navbar and mobile-menu language switcher with current-locale
  styling, accessible labels, and same-page switching.
- Preserved the current path, query string, and fragment when switching locale.
- Kept `/rest`, `/mcp`, `/swagger-ui`, sitemap routes, and JSON/CSV exports
  unprefixed.
- Translated global navigation, mobile controls, footer text, homepage metadata,
  hero, loading/error states, statistics headings, and the complete homepage
  manual.
- Added Chinese and Japanese homepage Markdown under `docs/i18n/`.
- Set `<html lang>` from the URL locale and added homepage canonical and
  `hreflang` metadata.
- Added `/zh-CN` and `/ja` to the static sitemap; localized table/detail routes
  are intentionally not advertised yet.
- Added tests for locale parsing, localized URL generation, URL-state
  preservation, utility/export exclusions, and localized sitemap entries.
- Verified formatting, Clippy, workspace tests, and the combined SSR/WASM
  `cargo leptos build`.

## Stable Rules

- The URL is the only locale source. Do not add cookie, browser-language, or
  `Accept-Language` redirects.
- Unprefixed routes always render English; do not introduce `/en/...` routes.
- Never translate catalog values, entity names, route identifiers, NASA field
  keys, SQL, commands, API paths, JSON/CSV keys, or scientific units.
- Keep stable homepage fragments identical across locales:
  `#mcp-exoplanet-data`, `#catalog-examples-title`, and `#mcp-setup-title`.
- Use locale resources for short interface strings and separate Markdown files
  for prose-heavy translated content.

## Deferred Translation Work

The following page-specific content still renders in English, even under locale
prefixes:

- table headings, filters, column selector, pagination, and table states
- stellar-host and exoplanet detail-page chrome
- insight overview/detail chrome and explanatory text
- not-found and shared application error/empty states
- technical docs content and docs registry metadata
- non-homepage localized metadata, canonical URLs, and `hreflang`

Do not add these partially translated routes to the sitemap. Add localized
sitemap entries only as each page surface is translated and receives correct
localized metadata.

## Recommended Next Pass

1. Translate shared table controls and states while leaving dataset values and
   column keys unchanged.
2. Ensure table navigation, pagination, filters, and entity links retain the
   active locale prefix and query state.
3. Translate detail and insight chrome.
4. Add localized docs summaries, then full translated docs only when reviewed.
5. Expand canonical, `hreflang`, and sitemap coverage alongside each completed
   surface.
