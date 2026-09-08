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

## Conversion Contract

[CLI conversion](../crates/exo-cli/src/conversion.rs) reads every `.vot` in
the chosen directory, writes Zstd-compressed Parquet beside it, and extracts
metadata into `<stem>-metadata.toml`. Existing outputs are overwritten.
Conversion reopens each Parquet file and checks its row and column counts.
The Justfile's `verify-data` recipe checks that the four expected runtime
files exist and are non-empty.

Startup reads the four runtime files from `EXO_DATA_DIR`, defaulting to
`data`. Uploaded files must be followed by restart/deployment before the
application uses them. VOTables are conversion inputs and are not uploaded
as runtime artifacts.

## Development Workflow

Use the [exodata-data skill](../.agents/skills/exodata-data/SKILL.md) for local
inspection, source-date queries, conversion, and the ordered NASA refresh
workflow. The [Justfile](../Justfile) owns download URLs and operational recipes.
[DEPLOY.md](../DEPLOY.md) remains the human-facing deployment guide.

Description selection operates on current local files; its date formats,
classification, output, and failure behavior are specified in
[cli.md](cli.md#description-regeneration-scan). Generation decisions remain
in the active task context.
