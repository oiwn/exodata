# Exoplanets Catalog Technical Overview

Exoplanets Catalog is an open-source technical data tool for exploring NASA Exoplanet Archive data through a server-rendered website, REST API, SQL query endpoint, and local CLI tooling.

^^^ data data 

The project is built in Rust. The web application uses Leptos for SSR and hydration, Axum for HTTP routing, Polars for in-memory columnar data queries, and Parquet files generated from NASA Exoplanet Archive VOTable exports.

## Public Surfaces

### Website

The website exposes the catalog through browsable, shareable routes:

- `/` - overview dashboard with catalog statistics
- `/stellarhosts` - stellar host table with pagination, sorting, filtering, and selectable columns
- `/stellarhosts/:hostname` - stellar host detail page
- `/exoplanets` - exoplanet table with pagination, sorting, filtering, and selectable columns
- `/exoplanets/:pl_name` - exoplanet detail page
- `/insights` - curated technical rankings and dataset views
- `/docs` - project, data source, and API summary
- `/swagger-ui` - interactive OpenAPI documentation

^^^ this should be valid relative urls, i want them to be rendered on server, maybe render component can generate links according to the router?
&&& Public routes are documented as root-relative paths (for example, `/exoplanets`). The Leptos router renders those routes during SSR; it does not generate link destinations from route declarations. UI links should use the existing locale-aware path helper so that a link preserves the active locale and URL state.

The table routes preserve query state in the URL so sorted, filtered, and column-customized views can be shared.


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

See `docs/api.md` for request parameters and examples.

### CLI

The workspace includes the `exodata` package in `crates/exo-cli`, which builds
the `exodata` public terminal client. It supports API-backed catalog access,
offline local data, downloads,
config, structured output, curated insights, and agent skill instructions.

Third-party oriented commands are top-level. Repository data preparation and
VOTable workflows live under `exodata dev`.

Examples:

```bash
cargo run -p exodata -- query "SELECT pl_name, hostname FROM exoplanets LIMIT 10"
cargo run -p exodata -- insights list
cargo run -p exodata -- insights run nearest-stellar-hosts
cargo run -p exodata -- dev view-metadata --path data/exoplanets.vot
```

See `docs/cli.md` and `specs/cli.md` for command details.

## Data Pipeline

The application uses two NASA Exoplanet Archive exports:

- `stellarhosts` - stellar host systems
- `pscomppars` - planetary systems/composite exoplanet parameters

Raw VOTable files are stored under `data/`, converted to Parquet, and loaded
into Polars DataFrames at server startup. Column metadata is extracted from the
VOTable source and stored as TOML so the website and API can expose names,
descriptions, units, and data types.

Runtime data files expected by the server:

```text
data/
├── stellarhosts.parquet
├── exoplanets.parquet
├── stellarhosts-metadata.toml
└── exoplanets-metadata.toml
```

See `specs/data-management.md` and `specs/column-metadata.md`.

## Architecture

At startup the SSR server:

1. Loads `stellarhosts.parquet` and `exoplanets.parquet` into shared
   `Arc<DataFrame>` values.
2. Loads TOML metadata for both tables.
3. Precomputes overview statistics.
4. Builds table, detail, and insight caches.
5. Builds sitemap XML from static routes, insight routes, and object detail
   routes.
6. Serves Leptos routes, REST API routes, Swagger UI, static assets, and
   sitemap index/child routes from the same Axum application.

The website uses Leptos server functions for UI data loading and the REST API
for external programmatic access. Both surfaces share the same in-memory data
and transformation code.

## Main Modules

```text
src/
├── app.rs                         # Leptos shell, routing, and layout
├── main.rs                        # Axum/Leptos server startup
├── server/
│   ├── handlers.rs                # REST API, OpenAPI, sitemap
│   ├── functions.rs               # Leptos server functions
│   ├── data/                      # table/detail/insight data access
│   └── cache.rs                   # runtime caches
├── table/                         # shared table state and query navigation
└── components/                    # website pages and UI components

crates/
├── exo-core/                      # data loading, metadata, insights, table logic
├── exo-cli/                       # exodata command-line package source
└── exo-types/                     # shared serializable types
```

## Documentation Map

Public and technical documentation:

- `docs/api.md` - REST API usage, SQL endpoint examples, response formats
- `docs/cli.md` - public CLI usage
- `docs/testing.md` - unit tests, e2e tests, and coverage
- `DEPLOY.md` - Docker, Ansible, DigitalOcean, and runtime deployment

Implementation specifications:

- `specs/web-backend.md` - Axum server, REST API, server functions, state
- `specs/web-frontend.md` - Leptos UI, routing, reactivity, hydration
- `specs/cli.md` - active CLI command specification
- `specs/data-management.md` - fetching, converting, and inspecting data
- `specs/column-metadata.md` - metadata extraction and API/schema exposure
- `specs/exoplanet-detail.md` - exoplanet detail page architecture
- `specs/stellarhost-details.md` - stellar host detail page notes
- `specs/styling.md` - component styling cleanup notes

Working notes and internal planning:

- `specs/ideas.md` - future ideas, including MCP follow-up
- `specs/refactoring.md` - refactoring notes
- `specs/architecture-cleanup-roadmap.md` - staged technical-debt and
  architecture cleanup plan
- `specs/ssr-streaming-issue.md` - production SSR streaming issue analysis
- `specs/ctx.md` - temporary active-task context; clean after use

## Development

Run the website locally:

```bash
cargo leptos watch
```

Open <http://127.0.0.1:3000>.

Build for production:

```bash
cargo leptos build --release
```

Run the CLI:

```bash
cargo run -p exodata -- --help
```

Run tests:

```bash
cargo test --workspace
```

## Deployment

Production deployment uses Docker images built by GitHub Actions and deployed
to a DigitalOcean droplet with Ansible.

Common commands:

```bash
just ansible-deploy
just ansible-status
just ansible-logs
```

See `DEPLOY.md` for the full deployment guide.

## Publishing Notes

This overview is intended to be suitable as public technical documentation.
Before publishing linked specs directly on the website, check each linked file
for stale route prefixes, old examples, temporary TODOs, and internal-only
working notes.
