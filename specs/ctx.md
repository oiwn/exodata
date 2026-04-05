# Current Context

## SEO Follow-Up

- Basic SEO groundwork is done: page metadata, `robots.txt`, and `sitemap.xml` are in place.

### Current Goal

- Continue the SEO pass by evaluating and adding useful JSON-LD / structured data.

### Implementation Choice

- Do not add a dedicated JSON-LD crate.
- Use `serde_json` to build schema objects directly and render them as JSON-LD.
- Reason: this task only needs emitting small schema.org payloads, not JSON-LD parsing, compaction, expansion, or RDF tooling.

### Priority

1. `WebSite` for overview
2. `CollectionPage` for `/stellarhosts` and `/exoplanets`
3. `Dataset` for stellar host and exoplanet detail pages

### Caution

- Only add schema types that honestly match the page and are likely to be useful to search engines.

### Quick Run Plan

1. Add a small helper module for structured data, likely `src/structured_data.rs`.
2. Keep the helper simple:
   - build `serde_json::Value`
   - serialize into `<script type="application/ld+json">`
3. Add `WebSite` JSON-LD to the overview page.
4. Add `CollectionPage` JSON-LD to:
   - `/stellarhosts`
   - `/exoplanets`
5. Add `Dataset` JSON-LD to:
   - stellar host detail pages
   - exoplanet detail pages
6. Evaluate whether extra schema such as `BreadcrumbList` is worth adding later.
7. Verify rendered page source for SSR output on:
   - `/`
   - `/stellarhosts`
   - `/exoplanets`
   - one stellar host detail page
   - one exoplanet detail page

### Likely Files To Edit

- `src/lib.rs`
- `src/structured_data.rs`
- `src/components/overview.rs`
- `src/components/stellarhosts_table.rs`
- `src/components/exoplanets_table.rs`
- `src/components/stellarhost_detail/page.rs`
- `src/components/exoplanet_detail.rs`
