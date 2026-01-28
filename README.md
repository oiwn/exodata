# Exoplanets Catalog

A web application for exploring the NASA Exoplanet Archive data. Browse stellar hosts and exoplanets through an interactive UI or query the data programmatically via REST API.

Built with Rust using [Leptos](https://github.com/leptos-rs/leptos) for the frontend, [Axum](https://github.com/tokio-rs/axum) for the backend, and [Polars](https://pola.rs/) for data processing.

For architecture details, see the [overview document](./specs/overview.md).

## Features

- **Interactive Web UI** - Browse and sort stellar hosts and exoplanets tables with customizable columns
- **REST API** - Paginated endpoints for stellarhosts and exoplanets with sorting and column selection
- **SQL Queries** - Execute SELECT queries directly against the dataset via API
- **Swagger Documentation** - Interactive API docs at `/swagger-ui`
- **Schema Introspection** - Get column metadata including descriptions and units

## Running the Application

```bash
cargo leptos watch
```

Open your browser to `http://127.0.0.1:3000`.

## REST API

Base URL: `/rest`

### Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /rest/stellarhosts` | Paginated stellar hosts data |
| `GET /rest/exoplanets` | Paginated exoplanets data |
| `GET /rest/stellarhosts/schema` | Column metadata for stellar hosts |
| `GET /rest/exoplanets/schema` | Column metadata for exoplanets |
| `GET /rest/query?sql=...` | Execute SQL SELECT queries |

### Query Parameters

- `page` - Page number (default: 1)
- `limit` - Rows per page (default: 50, max: 1000)
- `sort_by` - Column name to sort by
- `order` - Sort order: `asc` or `desc`
- `columns` - Comma-separated list of columns to return

### SQL Query Examples

```bash
# Get 10 exoplanets discovered by transit method
curl "http://localhost:3000/rest/query?sql=SELECT pl_name, hostname, disc_year FROM exoplanets WHERE discoverymethod = 'Transit' LIMIT 10"

# Join stellar hosts with their planets
curl "http://localhost:3000/rest/query?sql=SELECT s.hostname, s.st_teff, e.pl_name FROM stellarhosts s JOIN exoplanets e ON s.hostname = e.hostname LIMIT 10"
```

Available tables: `stellarhosts`, `exoplanets`

### Swagger UI

Interactive API documentation available at: `http://localhost:3000/swagger-ui`

## CLI Tools

The project includes command-line tools (`exo-cli`) for data exploration and metadata inspection.

### View Column Metadata

```bash
# View all column metadata from exoplanets VOTable
cargo run -p exo-cli -- view-metadata --path data/exoplanets.vot

# View metadata for specific columns only
cargo run -p exo-cli -- view-metadata --path data/exoplanets.vot --columns "pl_name,pl_orbper,pl_rade,pl_bmasse"

# View metadata from stellar hosts VOTable
cargo run -p exo-cli -- view-metadata --path data/stellarhosts.vot
```

### Other CLI Commands

```bash
# View field information from VOTable
cargo run -p exo-cli -- view-fields data/exoplanets.vot

# View sample data
cargo run -p exo-cli -- view-samples --limit 10
cargo run -p exo-cli -- view-exoplanets-samples --limit 10

# View statistics
cargo run -p exo-cli -- view-stats
cargo run -p exo-cli -- view-exoplanets-stats

# Convert VOTable files to Parquet
cargo run -p exo-cli -- convert-raw-files --data-dir data
```

## Testing

```bash
# Run all CLI tests
cargo test -p exo-cli

# Run tests with output
cargo test -p exo-cli -- --nocapture
```
