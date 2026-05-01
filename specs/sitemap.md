# Sitemap Specification

## Structure

The sitemap is split into a **sitemap index** and three **child sitemaps**, each served by Axum with `application/xml; charset=utf-8`:

| Route | Content |
|---|---|
| `GET /sitemap-index.xml` | `<sitemapindex>` listing 3 child sitemaps |
| `GET /sitemap-static.xml` | Static pages + insight pages (~16 URLs) |
| `GET /sitemap-stellarhosts.xml` | All `/stellarhosts/:hostname` detail pages |
| `GET /sitemap-exoplanets.xml` | All `/exoplanets/:pl_name` detail pages |

`public/robots.txt` points to `https://exodata.space/sitemap-index.xml`.

### Sitemap Index (`/sitemap-index.xml`)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap>
    <loc>https://exodata.space/sitemap-static.xml</loc>
    <lastmod>2026-05-01</lastmod>
  </sitemap>
  <sitemap>
    <loc>https://exodata.space/sitemap-stellarhosts.xml</loc>
    <lastmod>2026-05-01</lastmod>
  </sitemap>
  <sitemap>
    <loc>https://exodata.space/sitemap-exoplanets.xml</loc>
    <lastmod>2026-05-01</lastmod>
  </sitemap>
</sitemapindex>
```

### Child Sitemap Example (`/sitemap-exoplanets.xml`)

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
| `sitemap-stellarhosts.xml` | All unique `hostname` values from `stellarhosts_df` |
| `sitemap-exoplanets.xml` | All unique `pl_name` values from `exoplanets_df` |

## `<lastmod>` Value

Every `<url>` and `<sitemap>` entry carries `<lastmod>` in `YYYY-MM-DD` format. The date comes from:

1. `BUILD_DATE` env var if set (production: passed via Docker build arg from CI, format `YYYY-MM-DD`)
2. `BUILD_TIMESTAMP` env var if set (format `YYYY-MM-DD HH:MM UTC` — date portion extracted)
3. Current UTC date as fallback (local dev)

All entries share the same date — every URL is regenerated from scratch on each deploy.

See `compute_build_date()` in `src/main.rs`.

## Implementation

### Data Types

```rust
// src/server/handlers.rs
pub struct SitemapSet {
    pub index: String,
    pub static_pages: String,
    pub stellarhosts: String,
    pub exoplanets: String,
}

pub fn build_sitemaps(
    site_url: &str,
    build_date: &str,
    stellarhosts_df: &DataFrame,
    exoplanets_df: &DataFrame,
) -> Result<SitemapSet, String>
```

### State

`ApiState` holds four pre-rendered XML strings, built once at startup:

```rust
pub struct ApiState {
    // ...
    sitemap_index_xml: Arc<String>,
    sitemap_static_xml: Arc<String>,
    sitemap_stellarhosts_xml: Arc<String>,
    sitemap_exoplanets_xml: Arc<String>,
}
```

### Routes

```rust
pub fn site_routes(state: ApiState) -> Router {
    Router::new()
        .route("/sitemap-index.xml", get(get_sitemap_index))
        .route("/sitemap-static.xml", get(get_sitemap_static))
        .route("/sitemap-stellarhosts.xml", get(get_sitemap_stellarhosts))
        .route("/sitemap-exoplanets.xml", get(get_sitemap_exoplanets))
        .with_state(state)
}
```

### XML Formatting

Indented elements on separate lines:

```xml
  <url>
    <loc>...</loc>
    <lastmod>2026-05-01</lastmod>
  </url>
```

```xml
  <sitemap>
    <loc>...</loc>
    <lastmod>2026-05-01</lastmod>
  </sitemap>
```

### Build Pipeline

`deploy.yml` → `Dockerfile` → `main.rs`:

1. CI computes `BUILD_DATE=$(date -u +'%Y-%m-%d')` and passes it as Docker build arg
2. Dockerfile sets `ENV BUILD_DATE`
3. `main.rs` reads `option_env!("BUILD_DATE")` at compile time
4. `compute_build_date()` resolves the final date string
5. `build_sitemaps()` embeds it into all XML entries

## Files

| File | Role |
|---|---|
| `src/server/handlers.rs` | `SitemapSet`, `build_sitemaps`, `render_sitemap_index`, `render_urlset`, `build_detail_urls`, route handlers, `ApiState` |
| `src/main.rs` | `BUILD_DATE`/`BUILD_TIMESTAMP` constants, `compute_build_date`, startup wiring |
| `src/server/tests.rs` | Tests: `test_sitemap_index`, `test_sitemap_static`, `test_sitemap_stellarhosts`, `test_sitemap_exoplanets` |
| `public/robots.txt` | Points to `https://exodata.space/sitemap-index.xml` |
| `.github/workflows/deploy.yml` | Computes `BUILD_DATE` and `BUILD_TIMESTAMP`, passes as Docker build args |
| `infrastructure/docker/Dockerfile` | `ARG BUILD_DATE`, `ENV BUILD_DATE` |

## Out of Scope

- `<changefreq>` and `<priority>` — Google ignores both
- Gzip compression — not required at current sizes
- Google limits — well under 50,000 URLs per child sitemap and 50 MB per file
