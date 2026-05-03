# CLI Specification: exodata

`exodata` is the public CLI binary for Exoplanets Catalog. The package/crate
name remains `exo-cli`; the source folder is `crates/exo-cli`.

The CLI is primarily a third-party terminal client for the catalog API, with an
offline local-data backend for users who download the static data bundle.
Repository data engineering commands are grouped under `exodata dev`.

## Public Command Surface

Top-level commands intended for third-party users:

```text
exodata
├── query                    Execute SQL through the selected backend
├── rows                     Browse rows through the selected backend
├── schema                   View schema through the selected backend
├── insights                 List and run curated insight queries
├── download                 Download parquet and metadata files for offline use
├── config                   Read or update persistent CLI config
├── skill                    Print or install agent instructions
└── dev                      Repository/data-preparation commands
```

Global options:

```text
--backend <auto|api|local>
--api-base-url <URL>
--data-dir <PATH>
--output <table|json|csv>
```

## Installation And Publishing

The crates.io package name is `exo-cli`; the installed binary name is
`exodata`.

User installation after publish:

```bash
cargo install exo-cli
exodata --help
```

Repository installation:

```bash
cargo install --path crates/exo-cli
```

Crates.io publishing order:

```bash
cargo publish -p exo-types
cargo publish -p exo-core
cargo publish -p exo-cli
```

`exo-core` and `exo-cli` use versioned path dependencies for local workspace
development and crates.io publish compatibility. Their first `cargo package`
verification requires `exo-types` to exist in the crates.io index.

## Backend Model

Backend selection precedence:

1. `--backend <auto|api|local>`
2. `EXO_BACKEND`
3. config file `default_backend`
4. built-in default: `auto`

Backend behavior:

- `auto` uses a complete local dataset when available; otherwise it uses API.
- `api` always uses HTTP requests to the configured server.
- `local` requires a complete local dataset and reports missing files clearly.

Client-style local mode requires these files in one directory:

```text
stellarhosts.parquet
exoplanets.parquet
stellarhosts-metadata.toml
exoplanets-metadata.toml
```

Local dataset resolution:

1. `--data-dir` or `EXO_DATA_DIR`
2. config `local.data_dir`
3. `~/.exodata`

## Configuration

Config is stored outside the repository, using the platform config directory.

Initial config shape:

```toml
default_backend = "auto"

[api]
base_url = "https://exodata.space"
timeout_seconds = 30

[local]
data_dir = "~/.exodata"

[downloads]
directory = "~/.exodata"
overwrite = false

[output]
format = "table"
```

Environment variable overrides:

```text
EXO_BACKEND
EXO_API_BASE_URL
EXO_DATA_DIR
EXO_DOWNLOAD_DIR
```

## Public Commands

Backend-aware commands:

```bash
exodata query "SELECT pl_name, hostname FROM exoplanets LIMIT 10"
exodata rows exoplanets --columns pl_name,hostname,disc_year --limit 10
exodata schema exoplanets
exodata insights list
exodata insights run nearest-stellar-hosts
```

Offline data commands:

```bash
exodata download stellarhosts
exodata download exoplanets
exodata download all
```

Config and skill commands:

```bash
exodata config path
exodata config get default_backend
exodata config set default_backend local
exodata skill install local
exodata skill install global
```

Output formats:

- `--output table` (default)
- `--output json`
- `--output csv`

## Development Commands

Commands for repository maintainers and data preparation live under
`exodata dev`. They are compiled by default, but are not part of the primary
third-party user surface.

```text
exodata dev
├── view-fields
├── view-metadata
├── view-samples
├── view-stats
├── view-exoplanets-samples
├── view-exoplanets-stats
├── convert-raw-files
├── sql
└── insights run-all
```

Examples:

```bash
exodata dev view-fields data/exoplanets.vot
exodata dev view-metadata --path data/stellarhosts.vot
exodata dev convert-raw-files --data-dir data
exodata dev sql "SELECT pl_name, hostname FROM exoplanets LIMIT 10" --data-dir data
exodata dev insights run-all --data-dir data
```

The old top-level development command paths are intentionally removed. Existing
local scripts should migrate to `exodata dev ...`.

## REST And Agent Integration

API mode targets the public REST API documented in `docs/api.md`:

- `/rest/stellarhosts`
- `/rest/exoplanets`
- `/rest/stellarhosts/schema`
- `/rest/exoplanets/schema`
- `/rest/query`
- `/rest/insights`
- `/rest/insights/{slug}`

`exodata skill install local` writes
`.agents/skills/exodata/SKILL.md` under the current directory.
`exodata skill install global` writes
`~/.agents/skills/exodata/SKILL.md`.

The installed skill follows the Agent Skills directory convention and includes
an `installed-by: exodata` marker. Existing `exodata` installs are updated;
foreign/manual skill files are skipped.

A future MCP server should be built on top of REST endpoints, not local
parquet files. The initial planned MCP tools are:

- `list_insights()`
- `run_insight(slug)`

No MCP server is part of the current release.
