# Data Management

Covers fetching raw data from NASA and preparing it for the web app.

## Data Sources

NASA Exoplanet Archive TAP Service:

- **Stellar Hosts**: `https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+stellarhosts&format=votable`
- **Exoplanets (Planetary Systems, `ps`)**: `https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=select+*+from+ps&format=votable`

Use `ps`, which provides multiple reference-based parameter sets per planet,
including `default_flag`, `pl_refname`, `releasedate`, and `rowupdate`. Do not
substitute `pscomppars`: it combines parameters into one row per planet and
does not expose the same release/update fields. The application loads these
records under the local table name `exoplanets`.

`releasedate` identifies a parameter set's public release, while `rowupdate`
identifies its last planet-parameter update and may be missing. `pl_pubdate`
is the source publication date, not an archive update timestamp. The separate
`stellarhosts` table has no equivalent release/update fields.

See [NASA column definitions](https://exoplanetarchive.ipac.caltech.edu/docs/API_PS_columns.html).

## Directory Structure

```
data/
├── stellarhosts.vot
├── exoplanets.vot
├── stellarhosts.parquet
├── exoplanets.parquet
├── stellarhosts-metadata.toml
└── exoplanets-metadata.toml
```

VOTables are source files. The server loads both Parquet files and both
metadata TOML files at startup. Downloads and generated data files are
excluded from Git; snapshot retention, dataset diffs, and download versioning
are out of scope.

## Fetching Data

```bash
just download-data
```

The Justfile is the canonical download workflow. This command overwrites both
source VOTables. If the source files are already downloaded, start with conversion.

## Converting to Parquet

Reads every `.vot` file in `data/`, writes Zstd-compressed `.parquet` alongside
it, and extracts column metadata into `<stem>-metadata.toml`. Existing outputs
are overwritten. Remove temporary or obsolete VOTables before conversion.

```bash
just convert-raw-files
just verify-data
```

Conversion reopens each Parquet file and checks its row and column counts.
`verify-data` checks that all four runtime files exist and are non-empty.

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

# Inspect NASA date fields directly in the VOTable
cargo run --package exodata -- dev view-metadata --path data/exoplanets.vot --columns rowupdate,releasedate,pl_pubdate

# Sample rows from parquet
cargo run --package exodata -- dev view-samples
cargo run --package exodata -- dev view-exoplanets-samples

# Statistics
cargo run --package exodata -- dev view-stats
cargo run --package exodata -- dev view-exoplanets-stats

# Run SQL against parquet files (tables: stellarhosts, exoplanets)
cargo run --package exodata -- dev sql "SELECT pl_name, pl_orbper FROM exoplanets LIMIT 10"

# Inspect date values in the converted data
cargo run --package exodata -- dev sql "SELECT pl_name, hostname, rowupdate, releasedate FROM exoplanets ORDER BY rowupdate DESC LIMIT 10"
```

## Full Update

```bash
# 1. Download both NASA VOTables (skip if already downloaded)
just download-data

# 2. Convert both files and generate metadata
just convert-raw-files

# 3. Verify the four runtime files
just verify-data

# 4. Upload Parquet and metadata TOML files
just ansible-upload-data

# 5. Restart/deploy so the application loads the updated files
just ansible-deploy
```

Ansible commands require `infrastructure/ansible/.env` with `DROPLET_IP`
configured. Upload only generated Parquet/TOML files, not the VOTables.
See `DEPLOY.md` for deployment details.
