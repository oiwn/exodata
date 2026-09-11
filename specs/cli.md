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

### Bulk DeepSeek Generation

`exodata dev descriptions generate-batch` reads saved requests from immediate
subdirectories of `--input-dir` (default `content/systems`). Repeated `--hostname`
filters match exact request hostnames. Defaults: `--concurrency 4`,
`--max-tokens 1536`, and `--system-prompt content/stellarhost_prompt.txt`.
It never runs preparation or changes saved requests. `--force` regenerates
matching successful inputs. The existing single-request commands remain available.

Before network access, validate prepared schema version 1, hostname uniqueness,
matching local evidence, metadata identity, prompt, settings, and writable
output directories. Missing requested hosts and pending `.prepare`, `.generate`,
or `.fail-write` recovery artifacts fail preflight. Credentials are needed only
when calls remain after skip selection. One shared async client and a bounded
Tokio task set enforce concurrency. Model, thinking, timeout, streaming, redirects,
and retry settings match the single-call command. On 401/402/403/429 or storage
failure, stop scheduling but finish in-flight work. Other individual failures
do not stop the queue. Failed/not-started systems make the final exit nonzero.

The versioned SHA-256 fingerprint covers exact prompt text, parsed request
content, and generation settings. Recursively sorted object keys and preserved
array order/string contents make formatting and object order irrelevant. Send
deterministically serialized TOML. Credentials, timestamps, and returned usage
are not hash inputs. Matching metadata plus a nonempty description skips a call.

Successful results replace `description.md` and compact `metadata.toml` as a
pair with ordinary-error rollback, using `.generate` staging. Metadata includes
hostname, fingerprint/version, settings, timestamp, returned model, usage,
duration, the attempt count, and any recovered validation violations. A source
date is recorded only if present in the saved request; the legacy NASA-date
scanner remains separate from prepared-input fingerprinting. Success clears
old `fail.toml`. Failure preserves the previous successful pair and replaces
ignored `fail.toml` with attempted fingerprint/settings, timestamp,
error/status and available response body, finish reason, usage, duration, and
attempts. Non-success HTTP responses use sanitized diagnostics rather than raw
bodies; incomplete/invalid successful HTTP bodies are retained in the failure
record. Credentials are never persisted. No trial archive or root `tmp/`
workflow is used.

Generation is draft-critic-edit. Stage one (drafter) uses the shared system
prompt to turn the saved request into a factual draft, which is stored per
system as ignored `draft.md` once it validates. Stage two (critic) uses
`content/stellarhost_critic_prompt.txt` (override with `--critic-prompt`)
to verify the draft against the source facts; the call runs in DeepSeek
json mode (`response_format: json_object`) with a 768-token cap, returns
strict JSON findings (category/quote/problem/fix), gets one parse retry,
and fails open when unparseable or transport-failed, with a metadata note.
Stage three (editor) uses `content/stellarhost_editor_prompt.txt`
(override with `--editor-prompt`) and receives the draft, any critic
findings as defects to resolve, plus the full evidence request; it polishes
prose while preserving the science. Every response passes deterministic
gates before installation: one leading level-one title, paragraphs only
(no links, lists, tables, code, or extra headings), no em dashes or double
hyphens, every planet-name occurrence bolded, every used measurement phrase
(number plus unit, and the spectral label) bolded, output numeric tokens
within the request-licensed set (display strings, comparisons, guide
values, years, counts, object names - never raw audit values or errors),
and no banned phrasings (orbital position/speed, significance/hype,
missing-field commentary, raw-uncertainty derivation, the doubled hedge
"about about", meta/preparation language such as "the guide", "supplied",
"the request", or "the evidence", and unlicensed labels). Label bans are
licensing-aware: phrases licensed by prepared facts (a per-planet
`circumbinary` flag, star `host_kind = "pulsar"` and the `Pulsar Timing`
method label, star `age_class = "very young"`, and "sun-like" only for a
supplied G-type spectral label) are stripped before the ban list runs.
Stage outputs pass an algorithmic formatting pass before the gates:
plain number+unit measurement phrases (including full
`times Earth's` / `times Jupiter's` forms) and the supplied spectral
label are bolded automatically outside strong spans, and `times Earth`
is repaired to `times Earth's`; the bolding gate stays as a backstop.
Each drafter/editor stage retries with violations appended as formatting
feedback, at most three attempts per stage; transport/HTTP failures are
never retried. Exhausted retries fail the system naming the failed stage
while preserving the previous description. Metadata records per-stage
attempts, token usage, critic findings, plus totals; the fingerprint
(version 4) covers the request, all three prompt texts, and settings.

An optional hand-edited `notes.toml` may live beside the request:
`facts` entries merge into `publishable_comparisons` (their numbers
license reader-facing use) and `guidance` entries merge into
`silent_constraints`. The merge happens in memory at preflight and is
covered by the fingerprint, so edited notes regenerate the system.
Prepare and batch never write this file. `--failed` limits the batch to
systems with a `fail.toml` and retries them even when a matching
fingerprint would otherwise skip (recovering forced-rerun failures).

`dev descriptions status` summarizes a content directory without model
calls: per-system state (`failed` when a `fail.toml` records the latest
failed attempt, else `generated` for a nonempty description, else
`missing`), fingerprint version, attempts, token totals, critic findings
count, generated timestamp, and the failure error, with an aggregate
summary on stderr.

`dev descriptions normalize` re-applies the algorithmic formatting pass
(bolding and the density possessive) to stored descriptions without
model calls, rewriting only files that change.

`generate-batch`, `status`, `prepare`, and `normalize` default to the
`lines` output format: one compact single-line row per system (hostname,
state, attempts, short token count, duration or failure reason). Pass
`--output table`, `--output json`, or `--output csv` for the previous
renderings.

Progress and aggregate reported tokens/wall time go to stderr; stdout uses
the per-system output above. Generation success is a transport/
response check, not a factual correctness guarantee; no editorial states exist.

### Stellar-Host Input Preparation

```bash
exodata dev descriptions prepare --hostname "LHS 1140"
exodata dev descriptions prepare --hostname "LHS 1140" --hostname "TRAPPIST-1" --force
exodata dev descriptions prepare --hostname "LHS 1140" --data-dir data --output-dir content/systems --force
```

Offline preparation reads the two local Parquet files (`--data-dir`, default
`data`) and writes `evidence.json` and `request.toml` under
`<output-dir>/<system-id>/`. Output defaults to `content/systems`. One or
more exact NASA hostnames are required; repeat `--hostname` to prepare
several systems in one invocation. A per-system failure does not stop the
remaining hostnames; errors are reported per row (stderr, or an `error` row
in JSON) and the process exits nonzero when any preparation failed. No API,
download, or generation metadata writes occur. Preparation progress goes to
stderr ("Preparing/Prepared {hostname}") with each diagnostic on its own
indented line; table and CSV output show hostname, paths, and a diagnostics
count, while JSON rows carry the full diagnostics array.

Identifiers lowercase ASCII letters, replace whitespace with dashes, remove
characters other than ASCII letters/digits/dashes, collapse dashes, and trim
edge dashes. Empty identifiers and distinct catalog hostnames with the same
identifier fail. Existing preparation files require `--force`; stored hostname
mismatches fail even with force. Articles and `metadata.toml` remain untouched.
Inputs are validated and serialized before output files are written.
Evidence is generated local data ignored by Git, as is everything else
under `content/systems/`; requests and shared prompts at `content/` root
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
Missing-field diagnostics are retained in local evidence and CLI output, not
appended to model instructions. Minimum-mass and transit guide entries are
included only when relevant to the selected planets; the `spectral` entry
appears only with a stellar spectral type and `planet_classes` only when at
least one planet carries an estimate radius classification (Sub-Earth,
Earth-like, Super-Earth, Neptune-like, or Jupiter-like from the shared
`exo-core` radius classes; the per-planet `classification` field uses the
same thresholds). Mass display wording explicitly distinguishes a `Mass`
quantity from `Msini`. Planets with an estimate `Mass`-provenance mass and
an estimate radius also carry a precalculated mean-density fact from
ρ = M/R³ with Earth at 5.51 g/cm³; Msini planets and bounds are skipped.

Comparisons cover stellar mass/radius against solar units, stellar
temperature against the Sun's 5772 K, planetary radius and mass against
Earth, Jupiter-scale context for planetary masses clearly exceeding
Jupiter's 317.8 Earth masses, periods against 365 days, and first- and
second-place extrema of reported period/radius estimates and of reported
masses. Extrema rank by point value: every listed planet must carry an
estimate (interval-bearing) measurement for the key, and a fact fires only
when its value differs in rounded significant digits from every other
ranked planet; the "by reported estimates" wording is the uncertainty
hedge. Second-place facts additionally require at least three measured
planets. Mass extrema also require uniform provenance (`Mass` or `Msini`);
Msini sets are worded as minimum-mass quantities. Bounds and unknown
qualifiers suppress comparisons; overlapping supplied uncertainty
intervals suppress only baseline comparisons (against Earth, the Sun, or
Jupiter), not extrema. No deeper ordinals or new interplanetary ratios are
generated. Unknown mass provenance remains explicit.

The writing target is computed from fact richness: systems with fewer than
ten approved comparisons or a single planet target 150-300 words, others
300-600 words, always permitting shorter supported text.
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
