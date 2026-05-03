# Exoplanets Catalog

Exoplanets Catalog is an open-source data tool for exploring NASA Exoplanet Archive records. It provides a server-rendered website, REST API, SQL query endpoint, and CLI for working with stellar host and exoplanet datasets.

The project is built in Rust with Leptos, Axum, Polars and Clap. Source VOTable exports are converted to Parquet, loaded into memory at server startup, and served through interactive pages and programmatic endpoints.

## What You Can Do

- Browse [stellar host systems](/stellarhosts)
- Browse [confirmed exoplanets](/exoplanets)
- Open detail pages for individual hosts and planets from table rows
- Explore [curated dataset views](/insights)
- Query the catalog with the [REST API](api.md) and read-only SQL
- Inspect column metadata, units, and data types through [schema endpoints](api.md)
- Use the [CLI](cli.md) for API-backed queries, offline data, and curated insights

Table pages keep sorting, filtering, pagination, and selected columns in the URL so views can be shared or revisited.

## API

REST endpoints are mounted under `/rest`.

See [REST API](api.md) for parameters and examples.

## CLI

The `exo-cli` package builds the `exodata` terminal client:

- query the live REST API
- use downloaded local data offline
- inspect schemas and browse rows
- list and run curated insight queries
- run repository data-preparation commands under `exodata dev`

See [CLI Tools](cli.md) for commands and examples.

## Data Pipeline

The catalog uses two NASA Exoplanet Archive exports:

- Stellar Hosts: host star and system-level records
- Planetary Systems Composite Parameters: confirmed planet records and their host/system fields

Raw VOTable files are converted to Parquet for efficient columnar access. Column metadata is extracted from the source files and stored separately so the website and API can expose descriptions, units, and data types.

Runtime data files:

```text
data/
|-- stellarhosts.parquet
|-- exoplanets.parquet
|-- stellarhosts-metadata.toml
`-- exoplanets-metadata.toml
```

## Architecture

At startup, the server loads Parquet datasets and metadata into shared in-memory state. Axum serves the Leptos application, REST API, Swagger UI, static assets, and sitemap from one process.

Leptos server functions power the interactive website. REST endpoints expose the same underlying data for external tools and scripts. Polars provides DataFrame operations and SQL execution over the loaded catalog tables.

## Project Source

The project is open source:

<https://github.com/oiwn/exoplanets-catalog>
