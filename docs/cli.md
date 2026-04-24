# CLI Tools

Exoplanets Catalog includes a local CLI for working with VOTable source files, Parquet datasets, metadata, SQL queries, and curated insights.

Install from the repository:

```bash
cargo install --path crates/exo-cli
exodata --help
```

Build without installing:

```bash
cargo build -p exo-cli --release
./target/release/exodata --help
```

## Commands

Current commands:

```text
view-fields
view-metadata
view-samples
view-stats
view-exoplanets-samples
view-exoplanets-stats
convert-raw-files
sql
insights
```

## Data Files

By default, the CLI reads local files from `data/`:

```text
data/
|-- stellarhosts.vot
|-- exoplanets.vot
|-- stellarhosts.parquet
`-- exoplanets.parquet
```

## VOTable Commands

Inspect source files:

```bash
exodata view-fields data/exoplanets.vot
exodata view-metadata
exodata view-metadata --path data/stellarhosts.vot --columns "hostname,st_teff,st_mass"
```

## Sample Rows And Stats

Preview local parquet data:

```bash
exodata view-samples --limit 10
exodata view-samples --category stellar
exodata view-exoplanets-samples --category orbital
```

Show dataset statistics:

```bash
exodata view-stats
exodata view-exoplanets-stats
```

## Convert Raw Files

Convert `.vot` files in a directory to Parquet:

```bash
exodata convert-raw-files
exodata convert-raw-files --data-dir data
```

## SQL

Run SQL against local parquet files. Available tables:

- `stellarhosts`
- `exoplanets`

Examples:

```bash
exodata sql "SELECT pl_name, hostname, disc_year FROM exoplanets ORDER BY disc_year DESC LIMIT 10"
exodata sql "SELECT s.hostname, s.st_teff, e.pl_name FROM stellarhosts s JOIN exoplanets e ON s.hostname = e.hostname LIMIT 10"
```

Use `--data-dir` to point at another directory containing `stellarhosts.parquet` and `exoplanets.parquet`.

## Insights

List available insight slugs:

```bash
exodata insights list
```

Run one insight:

```bash
exodata insights run smallest-exoplanets-radius
exodata insights run nearest-stellar-hosts --data-dir data
```

Run all insights:

```bash
exodata insights run-all
```

## Related Docs

- [About](about.md)
- [REST API](api.md)
