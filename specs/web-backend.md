# Web Backend

The SSR application uses Axum, Leptos server functions, Polars, and in-memory
caches. Public request examples and wire formats live in
[API documentation](../docs/api.md) and [MCP documentation](../docs/mcp.md).

## Startup and State

[src/main.rs](../src/main.rs) starts a multithreaded Tokio runtime with four
worker threads. It loads both Parquet datasets and their metadata TOML files
from `EXO_DATA_DIR`, defaulting to `data`; missing or invalid files fail startup.
See [data-management.md](data-management.md) for artifact contracts.

Startup computes overview statistics, creates caches and sitemap XML, and
prewarms default table queries and registered insights before accepting requests.
The HTML shell embeds serialized metadata for hydration.

Startup also loads generated stellar-host prose: `EXO_CONTENT_DIR` (default
`content/systems`) is scanned once for `*/description.md` files, keyed by the
normalized system identifier and pre-rendered to HTML (leading level-one title
stripped; raw HTML characters escaped before Markdown parsing). A missing
content directory or per-system file is not an error: the map is simply empty
and affected pages render without the prose section. The `get_host_description`
server function normalizes the requested hostname via the shared
`exo_core::selection::system_identifier` and returns `None` when no
description exists.

`ApiState` in [handlers.rs](../src/server/handlers.rs) owns shared references to
the datasets, metadata, overview statistics, site URL, sitemap XML, and caches.
It is supplied to Axum handlers and Leptos server-function context.
`SITE_URL` defaults to `https://exodata.space`. Leptos configuration comes
from `Cargo.toml` locally or `LEPTOS_*` environment variables in deployment;
`LEPTOS_GA_ID` optionally enables analytics.

## Modules and Interfaces

- [server.rs](../src/server.rs) exposes server functions to both compilation
  targets and gates data, handlers, MCP, caches, and canonical calculations
  behind the `ssr` feature.
- [functions.rs](../src/server/functions.rs) owns serializable UI payloads;
  its `tables`, `details`, and `insights` children implement server functions.
- [data.rs](../src/server/data.rs) groups table queries, detail lookups, row
  conversion, summary transformations, insights, SQL, and exports.
- [handlers.rs](../src/server/handlers.rs) adapts shared data operations to
  REST, OpenAPI, sitemap routes, and detail export middleware.
- [mcp.rs](../src/server/mcp.rs) adapts the same server state and data operations
  to Streamable HTTP tools at `/mcp`.

REST lives under `/rest`; Leptos server-function transport is a separate
interface. Swagger UI is at `/swagger-ui`, with OpenAPI at
`/rest/openapi.json`. Website routes come from `src/app.rs`.

## Tables and Schema

Both REST table endpoints accept `page`, `limit`, `sort_by`, `order`,
`columns`, and `filter`. REST defaults to page 1 and 50 rows and caps
`limit` at 1,000. The shared backend normalizes page 0 to 1; website query
validation rejects invalid pages before fetching.

[Table processing](../src/server/data/tables.rs) selects valid requested
columns, or dataset-specific defaults. An empty valid selection is an error.
Filtering trims and lowercases the search text and performs substring matching
on the first selected column, casting it to strings when needed. Sorting
applies only to a selected column and removes rows with null sort values.
`desc` selects descending order; other values use ascending order.

`total_all` counts source rows before filtering/sorting; `total` counts the
remaining rows before pagination. Out-of-range backend requests return empty
rows; the website applies its own branded 404 behavior.

REST returns `{ data, columns, total, total_all, page, limit }`.
Leptos `TableData` uses `rows` instead of `data` and carries no metadata.
Schema endpoints return `columns` and `total_rows`, with each column's name,
Polars data type, and available description, unit, and source datatype.
See [column-metadata.md](column-metadata.md) for metadata ownership and UI flow.

## Caches and Overview Statistics

[cache.rs](../src/server/cache.rs) uses Moka caches for tables, stellar-host
details, and insights. Table keys include dataset, page, limit, sorting,
selected columns, and filter. Missing cached table queries run Polars work in
`spawn_blocking`; this is not a claim that all server data work is offloaded.

Startup capacities are set in `main.rs`. Host-detail entries are keyed by
hostname; insight entries by slug. Dataset files are loaded once, so replacing
runtime data requires restart. `get_stats` returns precomputed `DataStats`;
it does not recompute homepage aggregations per request.

## Detail Contracts

All payload types below are defined in
[functions.rs](../src/server/functions.rs). Lookups use exact catalog names
and retain matching source records; missing entities produce errors.

`StellarHostDetail` contains:

| Field | Content |
| --- | --- |
| `hostname`, `identity` | Exact hostname and identifier aliases |
| `system` | Planet/star/moon counts, distance, and parallax summaries |
| `star` | Spectral type, temperature, mass, radius, age, luminosity, metallicity, and gravity summaries |
| `provenance` | Record count, stellar/system references, and per-field measurement/distinct counts |
| `records`, `provenance_columns` | Source rows and selected provenance column names |
| `metadata` | Column metadata attached to the returned payload |

Host canonical values come from one source row selected by shared `exo-core`
logic. Score populated `st_spectype`, `st_teff`, `st_mass`, `st_rad`, `st_age`,
`st_lum`, `st_met`, `st_logg`, `sy_dist`, and `sy_plx`; numeric values must be
finite and spectral type nonblank. Zero counts as populated. Ties use lexical
stellar reference, system reference, then canonical row serialization. A zero
maximum score yields no selection. `selected_record_index` identifies the row
in the detail payload's `records`, retaining its references and qualifiers.
Primary numeric, categorical, and system-count values come only from that row;
missing fields stay missing. Counts, alternatives, and min/max ranges remain
all-record diagnostics, separate from adopted values. Aliases use all records.
`disputed` indicates distinct values, not a significance or anomaly test.

The host cache stores the detail without its metadata map; server functions and
JSON exports attach metadata when returning it. Related planets are fetched
separately as `HostPlanets { hostname, planets, columns, metadata }`.
This query groups by `pl_name` and uses each planet's unique `default_flag = 1`
row. Without a unique default, only its name is returned and measurements remain
unavailable. Planet detail payloads likewise expose `selected_record_index`.

`ExoplanetDetail { selected_record_index, pl_name, canonical, records, metadata }` is built from
matching planet records. See [exoplanet-detail.md](exoplanet-detail.md) for
summary fields and the current page's consumption of them.

## SQL, Insights, MCP, and Exports

`GET /rest/query` uses shared SELECT-only validation and Polars execution
against `stellarhosts` and `exoplanets`. It defaults to 1,000 returned rows,
caps at 10,000, and waits at most 30 seconds for the blocking task. A timeout
ends the request wait; it does not guarantee cancellation of running Polars work.

Curated insights use the registry and SQL definitions in `exo-core`, with
shared metadata in `exo-types`. REST exposes list/run endpoints, Leptos uses
server functions, and MCP exposes tools over the same server data.

MCP tools are `health`, `list_insights`, `run_insight`, `describe_catalog`,
`query_catalog`, and `download_detail`. MCP SQL defaults to 100 rows and
caps at 1,000; it shares validation and execution with REST.

Detail export middleware recognizes unprefixed host/planet URLs ending in
`.json` or `.csv`. JSON contains the detail payload; CSV contains matching
source rows. Responses include download filenames and content types.
Missing entities return 404. MCP `download_detail` uses the same export
functions and returns filename, MIME type, content, and URL.

REST table processing failures currently map to 500. SQL validation and
execution failures map to 400, timeouts to 408, and join failures to 500. Unknown insight
slugs return 404. Server functions return `ServerFnError`; page components
decide presentation. These behaviors are distinct from website route errors.

## Sitemaps

Sitemap XML is generated once at server startup and served from memory.
`/sitemap-index.xml` lists `/sitemap-static.xml` and entity chunks named
`/sitemap-stellarhosts-N.xml` and `/sitemap-exoplanets-N.xml`. Entity chunks
contain at most 1,000 URLs each, with deterministic numbering starting at 1,
using sorted, unique hostnames or planet names from the loaded datasets.
`public/robots.txt` points to the sitemap index.

The static sitemap includes `/`, `/zh-CN`, `/ja`, `/docs`, `/docs/cli`,
`/docs/api`, `/stellarhosts`, `/exoplanets`, `/insights`, and registered
`/insights/:slug` pages. Localized table/detail routes are not advertised;
see [localization.md](localization.md) for translation and metadata eligibility.
`/docs/mcp` and `/about` are currently absent from the sitemap; their intended
inclusion remains proposed in [ideas.md](ideas.md#routing-and-documentation).

All entries share a `YYYY-MM-DD` `<lastmod>` date selected by
`compute_build_date()` in `src/main.rs`: the build-time `BUILD_DATE` value,
then the date portion of `BUILD_TIMESTAMP`, then the current UTC date at
startup as the local-development fallback. This is a build/startup date,
not a per-record source-update timestamp.

Only generated entity chunk routes are registered. Unknown chunks and the
old unchunked `/sitemap-stellarhosts.xml` and `/sitemap-exoplanets.xml` return
404; no compatibility aliases are provided.

Implementation lives in `src/server/handlers.rs`, with startup wiring in
`src/main.rs`. Encoding, deduplication, chunking, and route behavior are covered
by the existing tests in `src/server/tests.rs`.


## Verification

See [testing.md](testing.md) for coverage and the
[checks skill](../.agents/skills/exodata-checks/SKILL.md) for execution.
Development data inspection uses the
[data skill](../.agents/skills/exodata-data/SKILL.md); deployment remains
documented in [DEPLOY.md](../DEPLOY.md).
