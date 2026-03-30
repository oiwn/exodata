# CLI Tools

The project includes `exo-cli` for data exploration and metadata inspection.

All commands run via:

```bash
cargo run -p exo-cli -- <command>
```

## Commands

### view-metadata

View column metadata from a VOTable file.

```bash
# All columns
cargo run -p exo-cli -- view-metadata --path data/exoplanets.vot

# Specific columns
cargo run -p exo-cli -- view-metadata --path data/exoplanets.vot --columns "pl_name,pl_orbper,pl_rade,pl_bmasse"

# Stellar hosts
cargo run -p exo-cli -- view-metadata --path data/stellarhosts.vot
```

### view-fields

Display field definitions from a VOTable file.

```bash
cargo run -p exo-cli -- view-fields data/exoplanets.vot
```

### sql

Execute SQL queries against parquet files using Polars `SQLContext`.

```bash
cargo run -p exo-cli -- sql "SELECT hostname, COUNT(*) AS rows FROM stellarhosts WHERE LOWER(hostname) LIKE '%gliese%' GROUP BY hostname ORDER BY rows DESC"
```

Available tables: `stellarhosts`, `exoplanets`

### view-samples / view-exoplanets-samples

View sample rows from the parquet files.

```bash
cargo run -p exo-cli -- view-samples --limit 10
cargo run -p exo-cli -- view-exoplanets-samples --limit 10
```

### view-stats / view-exoplanets-stats

Display statistics for each dataset.

```bash
cargo run -p exo-cli -- view-stats
cargo run -p exo-cli -- view-exoplanets-stats
```

### convert-raw-files

Convert VOTable (`.vot`) files to Parquet format.

```bash
cargo run -p exo-cli -- convert-raw-files --data-dir data
```
