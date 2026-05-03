# CLI

`exodata` is the terminal client for Exoplanets Catalog. It can query the live
API, use downloaded local data offline, and print table, JSON, or CSV output.

Install from crates.io:

```bash
cargo install exodata
exodata --help
```

Install from the repository:

```bash
cargo install --path crates/exo-cli
exodata --help
```

Build without installing:

```bash
cargo build -p exodata --release
./target/release/exodata --help
```

## Public Commands

```text
query
rows
schema
insights
download
config
skill
dev
```

Use `--backend api` for the live service, `--backend local` for a complete
local dataset, or the default `--backend auto` to use local data when available
and otherwise use the API.

Useful global options:

```text
--backend <auto|api|local>
--api-base-url <URL>
--data-dir <PATH>
--output <table|json|csv>
```

## Query Data

Run SQL:

```bash
exodata query "SELECT pl_name, hostname FROM exoplanets LIMIT 10"
exodata query "SELECT pl_name, hostname, disc_year FROM exoplanets ORDER BY disc_year DESC LIMIT 10" --output json
```

Browse rows:

```bash
exodata rows exoplanets --columns pl_name,hostname,disc_year --limit 10
exodata rows stellarhosts --columns hostname,sy_dist,st_teff --sort sy_dist --order asc --limit 10
```

View schema metadata:

```bash
exodata schema exoplanets
exodata schema stellarhosts --output json
```

## Insights

List available insight slugs:

```bash
exodata insights list
```

Run one insight:

```bash
exodata insights run smallest-exoplanets-radius
exodata insights run nearest-stellar-hosts --output json
```

## Offline Data

Download the local data bundle:

```bash
exodata download all
```

The default offline data directory is `~/.exodata`:

```text
~/.exodata/
|-- stellarhosts.parquet
|-- exoplanets.parquet
|-- stellarhosts-metadata.toml
`-- exoplanets-metadata.toml
```

Use another directory explicitly:

```bash
exodata --backend local --data-dir data schema exoplanets
```

## Config

```bash
exodata config path
exodata config get default_backend
exodata config set default_backend local
```

## Agent Skill

Install the Agent Skill for the current project:

```bash
exodata skill install local
```

Install it globally for your user:

```bash
exodata skill install global
```

Local install writes `.agents/skills/exodata/SKILL.md` under the current
directory. Global install writes `~/.agents/skills/exodata/SKILL.md`.

## Development Commands

Repository maintainers can use `exodata dev` for VOTable inspection,
conversion, local parquet SQL, and bulk insight verification:

```bash
exodata dev view-fields data/exoplanets.vot
exodata dev view-metadata --path data/stellarhosts.vot --columns hostname,st_teff,st_mass
exodata dev convert-raw-files --data-dir data
exodata dev sql "SELECT pl_name, hostname FROM exoplanets LIMIT 10" --data-dir data
exodata dev insights run-all --data-dir data
```

These commands are intentionally grouped away from the public top-level command
surface because most third-party users do not need VOTable or data-preparation
workflows.

## Related Docs

- [About](about.md)
- [REST API](api.md)
