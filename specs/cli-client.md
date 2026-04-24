# CLI Client Specification

This document defines the target direction for `exo` as a client for
Exoplanets Catalog, not only as a local data utility.

`specs/cli.md` remains the reference for the current command set. This file
describes the next-stage CLI design: configuration, backend selection, remote
API access, local data access, and download workflows.

## Goal

`exo` should support two usage modes:

- **API mode** - query a live Exoplanets Catalog deployment over HTTP
- **Local mode** - work directly with local VOTable and Parquet files

The CLI should feel like one tool with two backends, not two separate tools.

## Product Definition

`exo` is the terminal client for Exoplanets Catalog.

It should be able to:

- query the public REST API
- inspect local archive exports
- run local SQL against Parquet files
- download and refresh raw source data
- expose a stable config model for repeatable use outside the repo

## Current State

Current behavior is local-data-first:

- VOTable inspection
- metadata inspection
- Parquet conversion
- local SQL
- local insight execution

This is useful for development and data preparation, but it is not yet a full
client for the deployed catalog.

## Target Backend Model

The CLI should support a backend choice:

- `api`
- `local`

Backend selection precedence:

1. explicit CLI flag
2. environment variable
3. config file
4. built-in default

Suggested global option:

```bash
exo --backend api ...
exo --backend local ...
```

Suggested environment variable:

```text
EXO_BACKEND
```

## Configuration

The CLI needs persistent configuration outside the repository.

### Config Location

Preferred config location:

```text
$XDG_CONFIG_HOME/exo/config.toml
```

Typical fallback on Linux/macOS:

```text
~/.config/exo/config.toml
```

The implementation should use platform-appropriate config directories on
systems that do not follow XDG by default.

### Config Shape

Suggested first version:

```toml
default_backend = "api"

[api]
base_url = "https://exodata.space"
timeout_seconds = 30

[local]
data_dir = "~/.local/share/exo/data"

[downloads]
directory = "~/.local/share/exo/downloads"
overwrite = false

[output]
format = "table"
color = "auto"
```

### Environment Variables

Suggested environment variable overrides:

```text
EXO_BACKEND
EXO_API_BASE_URL
EXO_DATA_DIR
EXO_DOWNLOAD_DIR
```

## API Client Behavior

In API mode, the CLI should communicate with a configured Exoplanets Catalog
deployment, defaulting to:

```text
https://exodata.space
```

The CLI should target the same public API documented in `docs/api.md`:

- `/rest/stellarhosts`
- `/rest/exoplanets`
- `/rest/stellarhosts/schema`
- `/rest/exoplanets/schema`
- `/rest/query`

API mode should become the default for commands that are naturally client-like,
especially:

- schema inspection
- row browsing
- remote SQL
- future remote insight execution

## Local Data Behavior

Local mode should remain available for offline and development workflows:

- inspect raw VOTable files
- inspect local metadata
- convert VOTable files to Parquet
- run SQL over local Parquet files
- run local insight queries

Local mode is still the correct backend for source-data engineering and
verification workflows.

## Command Direction

The command set should gradually evolve toward user-intent-oriented commands.

Candidate areas:

- `config`
- `schema`
- `query`
- `insights`
- `download`

Possible command shapes:

```text
exo config path
exo config get
exo schema stellarhosts
exo schema exoplanets
exo query rows exoplanets
exo query sql "SELECT ..."
exo insights list
exo insights run <slug>
exo download stellarhosts
exo download exoplanets
exo download all
```

This is a target direction, not a requirement to refactor the existing command
set immediately.

## Download Workflow

The CLI should include a first-class download tool for NASA archive sources.

Suggested commands:

```text
exo download stellarhosts
exo download exoplanets
exo download metadata
exo download all
```

Expected behavior:

- download source files to a configured directory
- create directories if needed
- support overwrite/force behavior
- report download destination clearly
- integrate cleanly with later conversion steps

Optional follow-up behavior:

- convert immediately after download
- refresh metadata files

## Progress Reporting

Long-running CLI operations should show progress.

Progress reporting should be used for:

- source file downloads
- VOTable to Parquet conversion
- running all insights
- any future multi-step refresh workflow

Progress UI should show:

- current operation
- bytes or items processed when available
- elapsed time
- transfer rate or throughput when available
- completion status

`indicatif` is the expected implementation tool for this work.

## Output Formats

The CLI should move toward a shared output model across commands.

Suggested output options:

- `table`
- `json`
- `csv`

Candidate config field:

```toml
[output]
format = "table"
```

Candidate flag:

```bash
exo ... --output json
```

## Phased Delivery

### Phase 1

- add config loading
- add backend selection
- add API base URL support
- add API-backed schema/query commands

### Phase 2

- add download commands
- add progress reporting for downloads
- add configured data/download directories

### Phase 3

- unify output modes
- expand remote insight support
- reorganize command groups where it improves usability

## Non-Goals

For the first pass of this direction:

- do not remove current local-only commands yet
- do not require a full command-tree redesign before API support lands
- do not block remote mode on advanced caching

## Summary

The target CLI model is:

- **website** for interactive browsing
- **REST API** for remote machine-readable access
- **CLI** as a terminal client over either remote API or local files

That makes `exo` a coherent public interface for Exoplanets Catalog instead of
only a repo-local utility.
