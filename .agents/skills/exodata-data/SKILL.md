---
name: exodata-data
description: >-
  Inspect local Exodata VOTable/Parquet data, metadata, and description-scan
  results, or perform an authorized repository data refresh using existing
  CLI and Justfile workflows. Use for repository data preparation and debugging,
  rather than general catalog queries by end users.
---

# Exodata Data

Run from the repository root. Read
[data-management.md](../../../specs/data-management.md) for source/artifact
contracts and [cli.md](../../../specs/cli.md#development-commands) for command
behavior. Inspect the [Justfile](../../../Justfile) before refresh operations;
do not reconstruct download URLs or deployment commands from memory.

## Select an Operation

An inspection request calls for read-only CLI commands. It does not authorize
a download, conversion, generated-content write, or deployment.
Use explicit paths and `exodata dev` commands for repository data so public
client backend/config defaults do not redirect the investigation to the API.

Inspect available inputs first:

```bash
rg --files --no-ignore data -g '*.vot' -g '*.parquet' -g '*-metadata.toml'
```

Use the supplied/current dataset for data questions. E2E fixtures are stable
samples and cannot establish current NASA values or completeness.

## Read-Only Inspection

Choose the relevant command; do not run the entire list by default:

```bash
cargo run --locked -p exodata -- dev view-fields data/exoplanets.vot
cargo run --locked -p exodata -- dev view-metadata --path data/exoplanets.vot --columns rowupdate,releasedate,pl_pubdate
cargo run --locked -p exodata -- dev view-exoplanets-samples --path data/exoplanets.parquet --limit 5
cargo run --locked -p exodata -- dev view-samples --path data/stellarhosts.parquet --limit 5
cargo run --locked -p exodata -- dev view-exoplanets-stats --path data/exoplanets.parquet
cargo run --locked -p exodata -- dev view-stats --path data/stellarhosts.parquet
cargo run --locked -p exodata -- dev sql "SELECT pl_name, hostname, rowupdate, releasedate FROM exoplanets LIMIT 10" --data-dir data
cargo run --locked -p exodata -- dev descriptions scan --data-dir data --content-dir content/systems --output json
```

Check source field names and types before writing a query. Use bounded SELECT
queries for inspection. The description scanner reads current local Parquet
and existing artifacts; its statuses and date precedence are defined in the
[scanner contract](../../../specs/cli.md#description-regeneration-scan).
It makes no model calls and does not generate content.

If required files are missing, report the missing input or use another supplied
input that answers the question. Do not silently download replacement data.

## Authorized Refresh

Before conversion, inventory all `.vot` files in `data/`. The converter
processes every one and overwrites generated outputs. Resolve unexpected,
temporary, or obsolete VOTables before proceeding; do not silently delete
unrelated files.

Follow the existing recipe sequence, stopping on failure:

1. `just download-data` downloads both source VOTables. Skip this when the
   requested sources are already downloaded.
2. `just convert-raw-files` converts them and generates metadata. It reopens
   Parquet outputs to validate row/column counts.
3. `just verify-data` checks the four runtime files exist and are non-empty.

Expected generated files are `stellarhosts.parquet`, `exoplanets.parquet`,
`stellarhosts-metadata.toml`, and `exoplanets-metadata.toml` under `data/`.
Do not continue to upload after failed conversion or verification; outputs may
be partial. Investigate the observed failure before retrying.

For an explicitly requested custom directory, the equivalent converter is:

```bash
cargo run --locked -p exodata -- dev convert-raw-files --data-dir /path/to/data
```

The root `verify-data` recipe checks `data/`; verify the four files in the
custom directory instead when using this variant.

## Authorized Upload and Restart

Only when the task includes updating deployed data, continue after successful
verification with:

```bash
just ansible-upload-data
just ansible-deploy
```

Ansible requires `infrastructure/ansible/.env` with `DROPLET_IP` configured.
Check configuration availability without printing credentials. Upload generated
Parquet/TOML files, not VOTables. The application loads data at startup, so
upload alone does not refresh the running app. Use `just ansible-status` and
`just ansible-logs` to inspect the result of an authorized deployment.

Use [DEPLOY.md](../../../DEPLOY.md) for operational details. Neither this skill
nor a successful local conversion grants additional deployment or Git permissions.
