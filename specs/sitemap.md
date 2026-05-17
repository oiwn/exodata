# Sitemap Specification

## Structure

The sitemap is split into a sitemap index, one static child sitemap, and
chunked entity child sitemaps. All sitemap XML is built once at startup and
served by Axum with `application/xml; charset=utf-8`.

| Route | Content |
|---|---|
| `GET /sitemap-index.xml` | `<sitemapindex>` listing the static sitemap and every generated entity chunk |
| `GET /sitemap-static.xml` | Static pages + insight pages |
| `GET /sitemap-stellarhosts-1.xml`, `GET /sitemap-stellarhosts-2.xml`, ... | Chunked `/stellarhosts/:hostname` detail pages |
| `GET /sitemap-exoplanets-1.xml`, `GET /sitemap-exoplanets-2.xml`, ... | Chunked `/exoplanets/:pl_name` detail pages |

`public/robots.txt` points to `https://exodata.space/sitemap-index.xml`.

The old unchunked entity routes are not compatibility aliases:

- `/sitemap-stellarhosts.xml`
- `/sitemap-exoplanets.xml`

### Sitemap Index (`/sitemap-index.xml`)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap>
    <loc>https://exodata.space/sitemap-static.xml</loc>
    <lastmod>2026-05-01</lastmod>
  </sitemap>
  <sitemap>
    <loc>https://exodata.space/sitemap-stellarhosts-1.xml</loc>
    <lastmod>2026-05-01</lastmod>
  </sitemap>
  <sitemap>
    <loc>https://exodata.space/sitemap-exoplanets-1.xml</loc>
    <lastmod>2026-05-01</lastmod>
  </sitemap>
</sitemapindex>
```

### Child Sitemap Example (`/sitemap-exoplanets-1.xml`)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://exodata.space/exoplanets/Kepler-22%20b</loc>
    <lastmod>2026-05-01</lastmod>
  </url>
</urlset>
```

### URL Assignment

| Sitemap | URLs |
|---|---|
| `sitemap-static.xml` | `/`, `/docs`, `/docs/cli`, `/docs/api`, `/stellarhosts`, `/exoplanets`, `/insights`, all `/insights/:slug` routes |
| `sitemap-stellarhosts-N.xml` | All unique `hostname` values from `stellarhosts_df`, chunked after sorting/dedupe |
| `sitemap-exoplanets-N.xml` | All unique `pl_name` values from `exoplanets_df`, chunked after sorting/dedupe |

Entity sitemap chunks use a fixed limit of 1,000 URLs per file. Chunk names are
1-indexed and deterministic.

## URL Encoding

Entity detail route values are encoded as path segments:

- ASCII letters, digits, `-`, `.`, `_`, and `~` remain readable.
- Spaces are encoded as `%20`.
- Path-breaking or reserved characters such as `/`, `?`, `#`, `%`, and `&`
  are percent-encoded.
- Rendered `<loc>` values are XML-escaped after URL construction.

Examples:

```text
51 Eri b -> /exoplanets/51%20Eri%20b
Kepler-55 b -> /exoplanets/Kepler-55%20b
```

## `<lastmod>` Value

Every `<url>` and `<sitemap>` entry carries `<lastmod>` in `YYYY-MM-DD` format.
The date comes from:

1. `BUILD_DATE` env var if set (production: passed via Docker build arg from
   CI, format `YYYY-MM-DD`)
2. `BUILD_TIMESTAMP` env var if set (format `YYYY-MM-DD HH:MM UTC` - date
   portion extracted)
3. Current UTC date as fallback (local dev)

All entries share the same date because every URL is regenerated from scratch
on each deploy.

See `compute_build_date()` in `src/main.rs`.

## Implementation

`SitemapSet` contains the rendered index, static sitemap, and chunked entity
sitemaps keyed by filename. `ApiState` stores the index/static XML directly and
stores entity chunks in a filename-to-XML map.

`site_routes()` registers exact Axum routes for `/sitemap-index.xml`,
`/sitemap-static.xml`, and each generated entity chunk. Unknown chunks and the
old unchunked entity filenames return `404`.

Build flow:

1. `build_sitemaps()` trims `SITE_URL`, builds static URLs, and extracts unique
   detail names from the two DataFrames.
2. Detail URLs are sorted and deduped with `BTreeSet`.
3. Entity URLs are encoded, chunked into 1,000-URL files, and rendered as
   `<urlset>` XML.
4. The sitemap index lists the static sitemap, stellar host chunks, and
   exoplanet chunks in deterministic order.

## Files

| File | Role |
|---|---|
| `src/server/handlers.rs` | `SitemapSet`, `build_sitemaps`, sitemap rendering, chunk route registration, route handlers, `ApiState` |
| `src/main.rs` | `BUILD_DATE`/`BUILD_TIMESTAMP` constants, `compute_build_date`, startup wiring |
| `src/server/tests.rs` | Sitemap route, chunking, dedupe, encoding, and 404 coverage |
| `public/robots.txt` | Points to `https://exodata.space/sitemap-index.xml` |
| `.github/workflows/deploy.yml` | Computes `BUILD_DATE` and `BUILD_TIMESTAMP`, passes as Docker build args |
| `infrastructure/docker/Dockerfile` | `ARG BUILD_DATE`, `ENV BUILD_DATE` |

## Out of Scope

- `<changefreq>` and `<priority>` - Google ignores both
- Gzip compression
- Compatibility aliases for the old unchunked entity sitemap filenames
