# CLI Specification: exodata

`exodata` is the public CLI binary for Exoplanets Catalog. The package/crate
name is `exodata`; the source folder remains `crates/exo-cli`.

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

The crates.io package name is `exodata`; the installed binary name is
`exodata`.

User installation after publish:

```bash
cargo install exodata
exodata --help
```

Repository installation:

```bash
cargo install --path crates/exo-cli
```

Crates.io publishing order:

```bash
cargo publish -p exodata-types
cargo publish -p exodata-core
cargo publish -p exodata
```

`exodata-core` and `exodata` use versioned path dependencies for local
workspace development and crates.io publish compatibility. Their first
`cargo package` verification requires `exodata-types` to exist in the crates.io
index. Dependency aliases keep Rust import names as `exo_core` and
`exo_types`.

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
├── descriptions scan
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

### Description Regeneration Scan

```bash
exodata dev descriptions scan
exodata dev descriptions scan --all
exodata dev descriptions scan --output json
exodata dev descriptions scan --data-dir data --content-dir content/systems
```

This synchronous, read-only command always operates locally. Paths default to
`data` and `content/systems`, relative to the working directory, as with other
development commands. Backend selection and configured client data paths do not
affect the scan. Only `hostname`, `rowupdate`, and `releasedate` are loaded from
`exoplanets.parquet`; all three must be string columns.

Group all planetary reference rows by exact hostname, without a default-row
filter. The current source date is the maximum across both date columns and
all rows. Null and blank dates are ignored; nonblank dates must be valid calendar
dates in exact `YYYY-MM-DD` or `YYYY-MM-DD HH:MM:SS` format. Validate the
complete source timestamp and use its calendar date for comparison; metadata
`source_date` remains strictly `YYYY-MM-DD`.

Read immediate system directories containing `metadata.toml` and
`description.md`. Match only the exact string `hostname` in metadata, never the
directory name. Parse optional string `source_date`; tolerate other fields.
`generated_at`, model settings, and `request.toml` do not affect classification.

```toml
hostname = "HD 189733"
source_date = "2026-07-09"
generated_at = "2026-09-07T10:00:00Z"
model = "deepseek-v4-flash"
```

Classification precedence:

1. `unknown`: invalid dates, invalid metadata, duplicate metadata hostnames, or
   unreadable artifacts. Duplicate paths are listed in the reason, with no
   single metadata path or recorded date selected.
2. `missing`: description or metadata is absent, or description is empty or
   whitespace-only, even when dates are unavailable.
3. `outdated`: current source date is later than recorded source date.
4. `current`: both dates exist and match.
5. `unknown`: either date is unavailable, or the current date is earlier.

Report columns are `hostname`, `status`, `recorded_source_date`,
`current_source_date`, `metadata_path`, and `reason`. Unavailable values are JSON
nulls (empty cells in table/CSV). A valid maximum of available dates remains
visible even when another date is malformed. Default output hides `current`;
`--all` includes it. Global output selection uses the shared table/JSON/CSV
renderer. Stdout contains only the report.

Rows sort by hostname. Unassignable metadata errors appear first with null
hostname, ordered by metadata path. Missing metadata cannot be matched by
directory name; the corresponding dataset host remains `missing`. Valid metadata
for hosts absent from current data is ignored. A nonexistent content root is
treated as no generated content and is never created.

Unreadable Parquet, incompatible columns, invalid dataset hostnames (null or
blank), and unreadable content-root listings fail the command. Per-system
errors produce `unknown` rows; completed reports return success regardless of
statuses. The scanner writes no artifacts and makes no network requests.

Generation must save the source date actually used only after successfully
saving the corresponding description. Directory identifier generation and
request schema belong to generation. This scan does not detect removed records,
same-date corrections, or independent stellar-host updates.

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

The hosted MCP server is mounted by the web service at `/mcp` and is built on
top of the server's in-memory catalog state, not local parquet files. The
server uses Streamable HTTP in stateless JSON response mode. Current MCP tools
are:

- `health()`
- `list_insights()`
- `run_insight(slug)`
- `describe_catalog(table, columns)`
- `query_catalog(sql, limit)`

The MCP surface is read-only. `query_catalog` accepts one SQL `SELECT`
statement, registers `stellarhosts` and `exoplanets`, defaults to 100 rows, and
caps MCP responses at 1000 rows. Agents should call `describe_catalog` before
writing SQL when column names, units, or data types are uncertain.
