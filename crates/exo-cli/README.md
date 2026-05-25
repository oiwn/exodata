# exodata

Terminal client for the [Exoplanets Catalog](https://exodata.space) — a
queryable mirror of the NASA Exoplanet Archive's stellar hosts and exoplanets
tables.

`exodata` runs SQL `SELECT` queries, browses rows, inspects schemas, and runs
curated insights against the live API or a downloaded local dataset. Output is
table, JSON, or CSV.

## Install

```bash
cargo install exodata
exodata --help
```

## First Commands

```bash
# Live API, default table output
exodata schema exoplanets
exodata rows exoplanets --columns pl_name,hostname,disc_year --limit 10
exodata query "SELECT pl_name, hostname FROM exoplanets ORDER BY disc_year DESC LIMIT 10"

# Curated insights
exodata insights list
exodata insights run nearest-stellar-hosts
```

## Backends

```text
--backend auto   use local dataset if present, otherwise API   (default)
--backend api    always hit https://exodata.space
--backend local  require a complete local dataset
```

Selection precedence: `--backend` flag > `EXO_BACKEND` env > config file >
default `auto`.

## Offline Mode

```bash
exodata download all          # downloads parquet + metadata into ~/.exodata
exodata --backend local schema exoplanets
exodata --backend local --data-dir /path/to/bundle query "SELECT ..."
```

Local mode expects all four files in one directory:

```text
stellarhosts.parquet
exoplanets.parquet
stellarhosts-metadata.toml
exoplanets-metadata.toml
```

## Output Formats

```bash
exodata schema exoplanets --output table   # default, pretty-printed
exodata schema exoplanets --output json    # for scripts and agents
exodata schema exoplanets --output csv
```

## Configuration

```bash
exodata config path
exodata config get default_backend
exodata config set default_backend local
```

Persistent config lives at the platform config directory (e.g.
`~/.config/exodata/config.toml`). Environment overrides: `EXO_BACKEND`,
`EXO_API_BASE_URL`, `EXO_DATA_DIR`, `EXO_DOWNLOAD_DIR`.

## Agent Integration

The catalog also exposes a hosted MCP server at `https://exodata.space/mcp`
(Streamable HTTP, stateless JSON) with five read-only tools: `health`,
`list_insights`, `run_insight`, `describe_catalog`, `query_catalog`. Most
MCP-aware agents can connect natively — for example, Claude Code:

```bash
claude mcp add --transport http exodata https://exodata.space/mcp
```

See [the MCP docs](https://github.com/oiwn/exoplanets-catalog/blob/main/docs/mcp.md#connecting-an-agent)
for Crush, OpenCode, and Codex CLI snippets.

If you prefer the CLI as an agent surface, install the bundled skill so coding
agents pick up `exodata` usage instructions:

```bash
exodata skill install local    # writes .agents/skills/exodata/SKILL.md
exodata skill install global   # writes ~/.agents/skills/exodata/SKILL.md
```

## Repository Commands

Maintenance and data-preparation commands (raw-file conversion, sample views,
local SQL against `data/`, etc.) are grouped under `exodata dev`. These are
not aimed at end users; see the [CLI spec](https://github.com/oiwn/exoplanets-catalog/blob/main/specs/cli.md#development-commands)
for the full list.

## More

- Web UI: <https://exodata.space>
- REST API: <https://exodata.space/swagger-ui>
- Source: <https://github.com/oiwn/exoplanets-catalog>
- Full CLI docs:
  <https://github.com/oiwn/exoplanets-catalog/blob/main/docs/cli.md>

## License

AGPL-3.0-only.
