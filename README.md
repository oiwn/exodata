# Exoplanets Catalog

This is a web application for browsing a catalog of exoplanets. It is built with Rust, using the [Leptos](https://github.com/leptos-rs/leptos) framework for the frontend and [Axum](https://github.com/tokio-rs/axum) for the backend.

For more details on the project's structure, design, and architecture, please see the [overview document](./specs/overview.md).

## Running the Application

To run the application in development mode, use the following command:

```bash
cargo leptos watch
```

Then, open your browser to `http://127.0.0.1:3000`.

## CLI Tools

The project includes command-line tools (`exo-cli`) for data exploration and metadata inspection.

### View Column Metadata

View metadata (descriptions, units, data types) extracted from VOTable files:

```bash
# View all column metadata from exoplanets VOTable
cargo run -p exo-cli -- view-metadata --path data/exoplanets.vot

# View metadata for specific columns only
cargo run -p exo-cli -- view-metadata --path data/exoplanets.vot --columns "pl_name,pl_orbper,pl_rade,pl_bmasse"

# View metadata from stellar hosts VOTable
cargo run -p exo-cli -- view-metadata --path data/stellarhosts.vot
```

**Example output:**
```
Column: pl_orbper
  Unit: day
  Data Type: Double

Column: pl_rade
  Unit: Rearth
  Data Type: Double

Column: pl_bmasse
  Description:  Planet Mass or Mass*sin(i) [Earth Mass]
  Unit: Mearth
  Data Type: Double
```

This extracts metadata directly from the NASA Exoplanet Archive VOTable files, including:
- Column descriptions (when available)
- Units of measurement
- Data types

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

### CLI Tests

Run tests for the CLI tool:

```bash
# Run all CLI tests
cargo test -p exo-cli

# Run tests with output
cargo test -p exo-cli -- --nocapture
```

The tests verify that CLI commands execute without crashing.
