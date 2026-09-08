# CLI Specification: exodata

`exodata` is the public CLI binary for Exoplanets Catalog. The package and binary
are named `exodata`; the Rust library is `exo_cli`, and the source folder
remains `crates/exo-cli`.

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
├── skill                    Install public catalog-query agent instructions
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

Config is stored at `~/.exodata/config.toml`, under the user's home directory.

Default config shape:

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

### DeepSeek Connectivity Probe

`exodata dev descriptions probe` sends one fixed prompt ("Reply with only OK.")
to `https://api.deepseek.com/chat/completions` with `deepseek-v4-flash`,
`max_tokens = 32`, thinking disabled, and streaming disabled. There are no
retries or redirects; the request timeout is 60 seconds. No catalog data is
sent and no artifacts are written.

Read credentials only from `DEEPSEEK_API_KEY`; missing/blank values fail before
network access. Do not log credentials or HTTP response bodies on failure.
No dotenv loading or persistent credential configuration is used.

Shared table/JSON/CSV output reports text, returned model, finish reason,
prompt/completion/total tokens, and elapsed milliseconds. HTTP failures,
timeouts, malformed responses, empty text, and finish reasons other than
`stop` fail the command. A live request is manually tested by the developer.

### Manual DeepSeek Generation

```bash
exodata dev descriptions generate --input request.toml --output json
exodata dev descriptions generate --input prompt.txt --max-tokens 512
```

Read the UTF-8 file verbatim and send it as a single user message, without
TOML parsing or schema validation. Optional `--system-prompt <file>` reads a
second UTF-8 file verbatim and places it in a `system` message before the user
message. Without that option, no system message is added. An unreadable or
blank system prompt fails before network access. Missing/unreadable
files and blank input fail before network access. `--max-tokens` must be
positive and defaults to 256 output tokens. Input tokens are billed separately.

Use the probe's model, environment credential, timeout, disabled thinking,
no-retry policy, response validation, and shared table/JSON/CSV report format.
Truncation (`finish_reason = "length"`) fails without retrying; the user may
explicitly rerun with a larger limit. No content artifacts are written.
This command supports manual prompt experiments; it is not batch generation.

### Stellar-Host Input Preparation

```bash
exodata dev descriptions prepare --hostname "LHS 1140"
exodata dev descriptions prepare --hostname "LHS 1140" --data-dir data --output-dir content/systems --force
```

Offline preparation reads the two local Parquet files (`--data-dir`, default
`data`) and writes `evidence.json` and `request.toml` under
`<output-dir>/<system-id>/`. Output defaults to `content/systems`. One exact
NASA hostname is required. No API, download, or generation metadata writes occur.

Identifiers lowercase ASCII letters, replace whitespace with dashes, remove
characters other than ASCII letters/digits/dashes, collapse dashes, and trim
edge dashes. Empty identifiers and distinct catalog hostnames with the same
identifier fail. Existing preparation files require `--force`; stored hostname
mismatches fail even with force. Articles and `metadata.toml` remain untouched.
Inputs are validated and serialized before output files are written.
Evidence is generated local data ignored by Git; requests and shared prompts
remain tracked. Always refresh both files through `prepare`, not independently.
`--force` replaces request edits as well as evidence. Both new files and backups
are staged in a temporary `.prepare` directory before installation. Installation
failure restores the previous pair, including previously absent files. If
rollback fails, retain backups there and report the recovery location. An
existing `.prepare` directory blocks another preparation; inspect it before
manual recovery/removal. This is ordinary-error rollback, not crash-safe storage.

The shared core selector chooses the fullest host summary row and a unique
default row per matching planet. Missing selection fails; missing measurements
within selected rows remain missing. Selected rows, including nulls, flags,
errors, references, source filenames, and selection counts are stored in JSON.
Planet order is exact-name order. The matching-host planet count is distinct
from the selected row's catalog system count.

The TOML request separates assignment, explanatory guide, publishable facts,
approved comparisons, audit context, and silent constraints. Measurements carry
source field, value, unit, qualifier, display text, and available errors.
Publishable fields cover identity/distance/counts, stellar spectral type,
temperature/mass/radius/age, and planetary discovery method/year, period,
radius/mass. Same-row `pl_masse` is the only fallback for missing `pl_bmasse`.
Bounds and mass provenance remain explicit. Estimates use three significant
digits; bounds are not rounded. Parsecs convert with factor 3.26156.
The source distance field has uncertainty companions but no limit flag, so it
is treated as an estimate. Other missing/unrecognized limit flags are marked
unspecified and suppress comparisons. Nonpositive or nonfinite measurements
are omitted from publishable fields and reported in diagnostics; their selected
source rows remain in evidence.

Comparisons cover stellar mass/radius against solar units, planetary radius
against Earth, periods against 365 days, and extrema of reported period/radius
estimates. Bounds, unknown qualifiers, overlapping supplied uncertainty
intervals, and rounded ties suppress comparisons. No mass ranking or new
interplanetary ratios are generated. Unknown mass provenance remains explicit.

The writing target stays 300–600 words, permitting shorter supported text.
`content/stellarhost_prompt.txt` is the single editable system prompt passed
to the existing generator with `--system-prompt`; preparation never copies it.
The reusable guide in `content/prompts/stellarhost_guide.toml` is compiled into
preparation and included in requests. Exact prompt capture belongs to generation
trial artifacts. CLI reports hostname, paths, and diagnostics through existing
table/JSON/CSV formats. Technical tests use synthetic data, not live counts.

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

API mode targets the public REST API documented in [docs/api.md](../docs/api.md):

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

The installed public skill is maintained at
[crates/exo-cli/skills/exodata.md](../crates/exo-cli/skills/exodata.md).
It serves catalog users; repository development skills have distinct names
and live under `.agents/skills/`. The `skill` command currently supports
installation, not printing instructions.

Hosted MCP belongs to the web service, not the CLI process. Its shared-state
integration is described in [web-backend.md](web-backend.md#sql-insights-mcp-and-exports);
the complete tool inventory, including `download_detail`, is maintained in
[docs/mcp.md](../docs/mcp.md).
