# Current Context: CLI Release Cleanup

## Summary

`exodata` is the public CLI binary for Exoplanets Catalog. The crates.io
package/crate name remains `exo-cli`, and the source folder is
`crates/exo-cli`.

The current release task is to make the CLI friendlier for third-party users,
finish documentation cleanup, and leave a clear MCP-server follow-up plan.

## Current State

- CLI client commands support backend selection with `auto`, `api`, and
  `local`.
- The built-in backend default is `auto`.
- Local client mode requires a complete four-file dataset:
  - `stellarhosts.parquet`
  - `exoplanets.parquet`
  - `stellarhosts-metadata.toml`
  - `exoplanets-metadata.toml`
- Installed/offline local data defaults to `~/.exodata`.
- `insights list` uses the compiled local insight registry unless `api` is
  explicitly selected.
- REST insight endpoints have been added locally:
  - `GET /rest/insights`
  - `GET /rest/insights/{slug}`
  Production may still return 404 until the deployed server is updated.

## Release Decisions

- Public top-level CLI commands should be third-party oriented:
  - `query`
  - `rows`
  - `schema`
  - `insights`
  - `download`
  - `config`
  - `skill`
  - `dev`
- VOTable, conversion, local parquet inspection, legacy local SQL, and
  bulk insight verification commands live under `exodata dev`.
- Old top-level development command paths are removed rather than kept as
  hidden aliases.
- Development commands stay compiled by default for this release; no Cargo
  feature flag is introduced yet.
- `specs/cli.md` is the active technical source of truth for CLI behavior.
- `docs/cli.md` is the public third-party CLI documentation.
- The crates.io package should be published as `exo-cli`; users install it
  with `cargo install exo-cli`, which provides the `exodata` binary.
- The former client-mode CLI spec has been merged into `specs/cli.md` and
  deleted.
- MCP server implementation is deferred. The planned MCP surface should be
  backed by REST, initially:
  - `list_insights()`
  - `run_insight(slug)`

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

Local dataset resolution:

1. `--data-dir` or `EXO_DATA_DIR`
2. config `local.data_dir`
3. `~/.exodata`

The CLI does not automatically move or copy files from repo `data/`. Use
`exodata download all` to populate `~/.exodata`, or pass `--data-dir data`
explicitly for client-style local commands.

## Implemented CLI Surfaces

Public commands:

```bash
exodata query "SELECT pl_name, hostname FROM exoplanets LIMIT 10"
exodata rows exoplanets --columns pl_name,hostname,disc_year --limit 10
exodata schema exoplanets
exodata insights list
exodata insights run nearest-stellar-hosts
exodata download all
exodata config path
exodata skill install local
exodata skill install global
```

Development commands:

```bash
exodata dev view-fields data/exoplanets.vot
exodata dev view-metadata --path data/stellarhosts.vot
exodata dev convert-raw-files --data-dir data
exodata dev sql "SELECT pl_name, hostname FROM exoplanets LIMIT 10" --data-dir data
exodata dev insights run-all --data-dir data
```

## Release Verification

- `exodata --help` shows public commands and `dev`.
- `exodata dev --help` shows data engineering commands.
- Old top-level dev commands fail.
- Public command smoke tests work with `--backend api` or a complete local
  dataset.
- REST insight endpoints work locally and are verified after deploy.

## Relevant Specs And Docs

- `specs/cli.md` - active CLI command specification
- `specs/data-management.md` - data pipeline
- `specs/column-metadata.md` - metadata extraction and API/schema exposure
- `specs/ideas.md` - future ideas, including MCP follow-up
- `docs/cli.md` - public CLI docs
- `docs/api.md` - public API docs
