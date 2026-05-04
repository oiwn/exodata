---
name: exodata
description: Use exodata to query Exoplanets Catalog through the public API or local downloaded data. Use when the user asks about exoplanets, stellar hosts, catalog schemas, SQL queries, or curated dataset insights.
installed-by: exodata
---

# exodata CLI Skill

Use `exodata` to query Exoplanets Catalog data from the public API or local
downloaded Parquet files.

## Common Commands

```bash
exodata --backend api schema exoplanets --output json
exodata --backend api rows exoplanets --columns pl_name,hostname,disc_year --sort disc_year --order desc --limit 10 --output json
exodata --backend api query "SELECT pl_name, hostname, disc_year FROM exoplanets ORDER BY disc_year DESC LIMIT 10" --output json
exodata --backend api insights list --output json
exodata --backend api insights run nearest-stellar-hosts --output json
```

Prefer `--backend api` for live catalog access. Prefer `--output json` when
another program or agent needs to parse results.

Use local mode only when the user has downloaded or provided a complete local
dataset:

```bash
exodata --backend local --data-dir <dir> schema exoplanets --output json
exodata --backend local --data-dir <dir> query "SELECT pl_name, hostname FROM exoplanets LIMIT 10" --output json
```

Install offline data with:

```bash
exodata download all
```
