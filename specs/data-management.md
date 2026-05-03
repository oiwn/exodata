# Data Management

Covers fetching raw data from NASA and preparing it for the web app.

## Data Sources

NASA Exoplanet Archive TAP Service:
- **Stellar Hosts**: `https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+stellarhosts&format=votable`
- **Exoplanets**: `https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+pscomppars&format=votable`

## Directory Structure

```
data/
├── stellarhosts.vot       # Downloaded VOTable
├── stellarhosts.parquet   # Converted, used by the web app
├── exoplanets.vot         # Downloaded VOTable
└── exoplanets.parquet     # Converted, used by the web app
```

## Fetching Data

```bash
curl -o data/stellarhosts.vot \
  "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+stellarhosts&format=votable"

curl -o data/exoplanets.vot \
  "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+pscomppars&format=votable"
```

## Converting to Parquet

Reads all `.vot` files in `data/` and writes `.parquet` alongside them:

```bash
cargo run --package exodata -- dev convert-raw-files
```

Or with a custom data directory:

```bash
cargo run --package exodata -- dev convert-raw-files --data-dir path/to/data
```

## Inspecting Data

```bash
# View VOTable column headers
cargo run --package exodata -- dev view-fields data/stellarhosts.vot

# View column metadata (units, descriptions)
cargo run --package exodata -- dev view-metadata --path data/exoplanets.vot

# Sample rows from parquet
cargo run --package exodata -- dev view-samples
cargo run --package exodata -- dev view-exoplanets-samples

# Statistics
cargo run --package exodata -- dev view-stats
cargo run --package exodata -- dev view-exoplanets-stats

# Run SQL against parquet files (tables: stellarhosts, exoplanets)
cargo run --package exodata -- dev sql "SELECT pl_name, pl_orbper FROM exoplanets LIMIT 10"
```

## Full Update

```bash
# 1. Download latest data
curl -o data/stellarhosts.vot "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+stellarhosts&format=votable"
curl -o data/exoplanets.vot "https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+pscomppars&format=votable"

# 2. Convert to parquet
cargo run --package exodata -- dev convert-raw-files
```
