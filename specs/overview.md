# Exoplanets Catalog Technical Overview

Exoplanets Catalog exposes NASA Exoplanet Archive data through a server-rendered
website, REST API, read-only MCP server, and CLI with API and local-data backends.

^^^ data data 

The project is built in Rust. The web application uses Leptos for SSR and hydration, Axum for HTTP routing, Polars for in-memory columnar data queries, and Parquet files generated from NASA Exoplanet Archive VOTable exports.

## Public Surfaces

### Website

The website exposes the catalog through browsable, shareable routes:

- [Overview](/) - dashboard with catalog statistics
- [Stellar hosts](/stellarhosts) - table with pagination, sorting, filtering, and selectable columns
- `/stellarhosts/:hostname` - stellar host detail page
- [Exoplanets](/exoplanets) - table with pagination, sorting, filtering, and selectable columns
- `/exoplanets/:pl_name` - exoplanet detail page
- [Insights](/insights) and `/insights/:slug` - curated rankings and dataset views
- [Docs](/docs), [CLI](/docs/cli), [API](/docs/api), and [MCP](/docs/mcp) - rendered public documentation
- [About](/about) - project information
- [Swagger UI](/swagger-ui) - interactive OpenAPI documentation

^^^ this should be valid relative urls, i want them to be rendered on server, maybe render component can generate links according to the router?
&&& Public routes are documented as root-relative paths (for example, `/exoplanets`). The Leptos router renders those routes during SSR; it does not generate link destinations from route declarations. UI links should use the existing locale-aware path helper so that a link preserves the active locale and URL state.

The table routes preserve query state in the URL so sorted, filtered, and column-customized views can be shared.

Website pages have English routes without a prefix and aliases under `/zh-CN`
and `/ja`. The URL selects the locale. Global navigation and homepage content
are translated; the remaining page-specific translation scope is tracked in
[localization.md](localization.md). REST, MCP, Swagger, sitemap, and detail
export URLs remain unprefixed.

Page routes use lazy-loaded components with `SsrMode::Async`, followed by WASM
hydration. Public Markdown pages are compiled from `docs/` through the registry
in `src/components/docs/registry.rs`.

### REST API

REST endpoints are mounted under `/rest`:

- `GET /rest/stellarhosts` - paginated stellar host rows
- `GET /rest/exoplanets` - paginated exoplanet rows
- `GET /rest/stellarhosts/schema` - stellar host column metadata
- `GET /rest/exoplanets/schema` - exoplanet column metadata
- `GET /rest/query?sql=SELECT...` - read-only SQL query endpoint
- `GET /rest/insights` - curated insight metadata
- `GET /rest/insights/{slug}` - curated insight results
- `GET /rest/openapi.json` - OpenAPI specification used by Swagger UI

Data endpoints support pagination, sorting, selected columns, and text
filtering. The SQL endpoint accepts a single `SELECT` statement, registers the
`stellarhosts` and `exoplanets` tables in Polars SQL, caps returned rows, and
applies a server-side timeout.

Detail pages also expose JSON and CSV downloads through
`/stellarhosts/:hostname.json`, `/stellarhosts/:hostname.csv`,
`/exoplanets/:pl_name.json`, and `/exoplanets/:pl_name.csv`.

See [API documentation](../docs/api.md) for request parameters and examples.

### MCP

The Axum application mounts a read-only Streamable HTTP MCP server at `/mcp`.
Its tools are `health`, `list_insights`, `run_insight`, `describe_catalog`,
`query_catalog`, and `download_detail`. It uses the server's in-memory catalog
state and shared SQL/export functions. See [MCP documentation](../docs/mcp.md)
for tool contracts and client configuration.

### CLI

The workspace includes the `exodata` package in `crates/exo-cli`, which builds
the `exodata` public terminal client. It supports API-backed catalog access,
offline local data, downloads, configuration, table/JSON/CSV output, curated
insights, and installation of a public catalog-query skill.

Third-party oriented commands are top-level. Repository data preparation and
VOTable workflows and the local description-regeneration scan live under
`exodata dev`.

Examples:

```bash
cargo run --locked -p exodata -- query "SELECT pl_name, hostname FROM exoplanets LIMIT 10"
cargo run --locked -p exodata -- insights list
cargo run --locked -p exodata -- insights run nearest-stellar-hosts
cargo run --locked -p exodata -- dev view-metadata --path data/exoplanets.vot
```

See [CLI documentation](../docs/cli.md) and [cli.md](cli.md) for command details.

## Data Pipeline

The application uses two NASA Exoplanet Archive exports:

- `stellarhosts` - stellar host systems
- `ps` - reference-based planetary-system records, stored locally as `exoplanets`

Raw VOTable files are stored under `data/`, converted to Parquet, and loaded
into Polars DataFrames at server startup. Column metadata is extracted from the
VOTable source and stored as TOML so the website and API can expose names,
descriptions, units, and data types. The CLI owns VOTable parsing and conversion;
`exo-core` owns Parquet loading and metadata persistence, and `exo-types` owns
the shared metadata type. Use the [Justfile](../Justfile) download and conversion
recipes; see [data-management.md](data-management.md) for the workflow.

Runtime data files expected by the server:

```text
data/
├── stellarhosts.parquet
├── exoplanets.parquet
├── stellarhosts-metadata.toml
└── exoplanets-metadata.toml
```

The server reads these four files from `EXO_DATA_DIR`, defaulting to `data`.
Missing or invalid runtime files prevent startup. Refreshing the files requires
a server restart to load them. See [column-metadata.md](column-metadata.md) for
the metadata specification.

## Architecture

At startup the SSR server:

1. Loads `stellarhosts.parquet` and `exoplanets.parquet` into shared
   `Arc<DataFrame>` values.
2. Loads TOML metadata for both tables and serializes it for the shared UI
   metadata store embedded in the HTML shell.
3. Precomputes overview statistics.
4. Creates table, stellar-host detail, and insight caches, then prewarms default
   table queries and registered insights before accepting requests.
5. Builds sitemap XML from static routes, insight routes, and object detail
   routes.
6. Serves Leptos routes, REST, MCP, detail exports, Swagger UI, static assets,
   and sitemap index/child routes from the same Axum application.

The website uses Leptos server functions for UI data loading. REST and MCP
share the in-memory catalog state and server data functions. The CLI's local
backend loads downloaded runtime files separately. The SSR runtime explicitly
uses four Tokio worker threads.

## Main Modules

```text
src/
├── app.rs                         # Leptos shell, routing, and layout
├── main.rs                        # Axum/Leptos server startup
├── locale.rs                      # URL locale selection and path helpers
├── metadata.rs                    # shared UI metadata store and hydration
├── server.rs                      # shared/server-only module boundaries
├── server/
│   ├── handlers.rs                # REST API, OpenAPI, sitemap
│   ├── mcp.rs                     # hosted MCP tools
│   ├── functions.rs + functions/  # shared payloads and Leptos server functions
│   ├── data.rs + data/            # tables, details, insights, SQL, exports
│   ├── cache.rs                   # runtime caches
│   ├── stellarhost_canonical.rs   # host record summaries
│   └── exoplanet_canonical.rs     # planet record summaries
├── table.rs + table/              # shared table state, navigation, rendering
└── components.rs + components/    # website pages and feature components

crates/
├── exo-core/                      # data loading, metadata, insights, table logic
├── exo-cli/                       # CLI backends, VOTable conversion, dev commands
└── exo-types/                     # shared serializable types
```

Feature styles live in `style/components/*.css` and are imported by
`style/tailwind.css`; see [frontend styling](web-frontend.md#styling-with-tailwind-css).

## Documentation Map

Public and technical documentation:

- [docs/about.md](../docs/about.md) - public project and data-access summary
- [docs/index.md](../docs/index.md) - homepage manual
- [docs/api.md](../docs/api.md) - REST API usage, SQL, response formats, exports
- [docs/cli.md](../docs/cli.md) - public CLI usage
- [docs/mcp.md](../docs/mcp.md) - hosted MCP tools and client configuration
- [DEPLOY.md](../DEPLOY.md) - Docker, Ansible, and runtime deployment

Implementation specifications:

- [web-backend.md](web-backend.md) - Axum, REST, server functions, state, sitemaps
- [web-frontend.md](web-frontend.md) - Leptos UI, routing, hydration, styling
- [cli.md](cli.md) - CLI contracts and development commands
- [data-management.md](data-management.md) - source files and runtime data workflow
- [column-metadata.md](column-metadata.md) - metadata extraction and schema exposure
- [exoplanet-detail.md](exoplanet-detail.md) - current and proposed detail-page contracts
- [localization.md](localization.md) - locale behavior and translation scope
- [testing.md](testing.md) - test coverage, fixtures, E2E setup, and verification

Working notes and internal planning:

- [ctx.md](ctx.md) - active task context and open decisions
- [roadmap.md](roadmap.md) - future work and issue-triage notes
- [ideas.md](ideas.md) - uncommitted product and development-harness ideas

## Development

Project workflows are documented in the
[checks skill](../.agents/skills/exodata-checks/SKILL.md) for focused validation
and the [data skill](../.agents/skills/exodata-data/SKILL.md) for local inspection
and authorized refresh operations. Technical contracts remain in the specs above.

Run the website locally:

```bash
cargo leptos watch --split
```

Open <http://127.0.0.1:3000>.

The four runtime data files must be available first. For fixture-based setup
and browser smoke tests, see [testing.md](testing.md). `--split` is required
for the application's lazy routes.

Build for production:

```bash
cargo leptos build --release --split
```

Run the CLI:

```bash
cargo run --locked -p exodata -- --help
```

Run tests:

```bash
cargo test --locked --workspace
```

### Dependency Policy

Canonical dependency requirements include major, minor, and patch components
and use Cargo's normal caret semantics, except for explicit compatibility pins.
`Cargo.lock` is committed. Use locked resolution for local checks; CI tests and
coverage use `--locked`, and Cargo Leptos passes it to both server and hydration
builds through `Cargo.toml`, including Docker builds. Dependency changes must
keep the manifests and lockfile consistent.

Cargo Audit configuration lives in `.cargo/audit.toml`. Yanked-crate checks
remain enabled, while unmaintained, unsound, and notice advisories are reported
without failing CI. `RUSTSEC-2026-0194` is accepted because VOTable conversion
reads trusted offline NASA files, and `RUSTSEC-2026-0195` is accepted because
the affected Polars cloud XML paths are not enabled or used. Other advisories
remain unsuppressed.

Serde is temporarily pinned to `1.0.228` because VOTable `0.7.0` imports that
release's private module; return it to a three-part caret requirement when the
VOTable dependency supports newer Serde releases.

## Deployment

Production deployment uses Docker images built by GitHub Actions and deployed
to a DigitalOcean droplet with Ansible.

Common commands:

```bash
just ansible-deploy
just ansible-status
just ansible-logs
```

See [DEPLOY.md](../DEPLOY.md) for the full deployment guide.

## Publishing Notes

The website currently publishes the selected `docs/` files registered in
`src/components/docs/registry.rs`; it does not publish this overview or all
specs automatically. Before adding a spec to public documentation, review its
technical accuracy, links, temporary notes, and unresolved remarks.
