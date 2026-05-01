# Current Context: CLI + REST API Expansion

^^^ i think cli name would be "exodata"
&&& agreed. The public binary name is `exodata`; `crates/exo-cli/Cargo.toml`
already declares `[[bin]] name = "exodata"`. The crate/package can remain
`exo-cli`, but CLI examples and user-facing docs should use `exodata`.

## Summary

Expand `exodata` from a repo-local data utility into a full terminal client for
Exoplanets Catalog. The CLI should support two backends (local files and remote
API), download static data for offline use, and expose an agentic skill so AI
agents can query the catalog without downloading parquet files themselves.

## What Exists

- `exo-cli` crate with local-only commands (view-fields, view-samples, sql,
  insights, convert-raw-files, etc.) — see `specs/cli.md`
- REST API under `/rest/` with Axum/utoipa handlers — see `docs/api.md` for current reference
^^^ i suggest web-backend heavily outdate, we should use separate axum/utoipa endpoints to build the API, currently leptos use server functions, but we'll not use them
&&& agreed. `specs/web-backend.md` documents an early design (Leptos server functions, `/api/` prefix, `QueryParams` with hardcoded filter fields) that no longer matches reality. The actual API uses Axum+utoipa handlers under `/rest/`, generic column-agnostic query params, cached table access, and OpenAPI docs via Swagger UI. The spec needs a full rewrite to reflect the current `src/server/handlers.rs` implementation. Marking `specs/web-backend.md` as stale — it should not be used as a reference. The source of truth is the code + `docs/api.md`.
- `specs/cli-client.md` — merged into this document (see sections below)
  ^^^ need to merge them
&&& done. `cli-client.md` content is already reflected in this document's Design Decisions, Implementation Plan, and Open Questions sections. The phase structure from `cli-client.md` is reproduced here as Phase 1–6. `specs/cli-client.md` can be deleted or left as archived reference.
- `specs/ideas.md` — agent/tool access to insights via REST, MCP, skill setup

## Design Decisions

### Data download via static files

The server will serve pre-built parquet and metadata files through nginx as
static assets (not generated on demand). The CLI downloads these into a local
config directory for offline use.

**Download directory**: `~/.exodata/` (cross-platform)

| Platform | Location |
|----------|----------|
| Linux | `$XDG_DATA_HOME/.exodata/` or `~/.exodata/` |
| macOS | `~/Library/Application Support/.exodata/` or `~/.exodata/` |
| Windows | `%APPDATA%\.exodata\` |

For simplicity, start with `~/.exodata/` on all platforms using the `dirs`
crate, then refine per-platform if needed. The dot-prefix keeps it out of sight
but discoverable.

Expected local structure after download:

```
~/.exodata/
├── stellarhosts.parquet
├── exoplanets.parquet
├── stellarhosts-metadata.toml
└── exoplanets-metadata.toml
```

**Static file serving** (nginx config or similar):

```
/data/exoplanets.parquet       → /static/exoplanets.parquet
/data/stellarhosts.parquet     → /static/stellarhosts.parquet
/data/exoplanets-metadata.toml → /static/exoplanets-metadata.toml
/data/stellarhosts-metadata.toml → /static/stellarhosts-metadata.toml
```

CLI download commands:

```bash
exodata download stellarhosts      # download parquet + metadata
exodata download exoplanets        # download parquet + metadata
exodata download all               # both datasets
```

### Two backend modes

| Mode | Source | When to use |
|------|--------|-------------|
| `api` | HTTP requests to configured server (default `https://exodata.space`) | Online, querying live data |
| `local` | Parquet files from `~/.exodata/` or `--data-dir` override | Offline, development, data engineering |

Backend selection precedence:

1. `--backend <api|local>` flag
2. `EXO_BACKEND` environment variable
3. config file (`default_backend` key)
4. built-in default: `api`

Commands that only make sense locally (view-fields, convert-raw-files) ignore
backend selection and always read local files.

### Output formats

All query/table commands accept `--output` / `-o`:

| Format | Flag | Description |
|--------|------|-------------|
| table | `--output table` (default) | Pretty-printed terminal table via comfy-table |
| json | `--output json` | NDJSON or JSON array to stdout |
| csv | `--output csv` | CSV with header row to stdout |

Config default:

```toml
[output]
format = "table"
```

### Query language

For table browsing, support three input styles:

1. **SQL** — `exodata query "SELECT pl_name, pl_rade FROM exoplanets ORDER BY pl_rade LIMIT 10"`
   Works in both modes. In API mode, hits `/rest/query?sql=...`. In local mode,
   runs Polars SQL against local parquet.

2. **Structured flags** — `exodata rows exoplanets --sort pl_rade --limit 10 --columns pl_name,pl_rade`
   Translates to REST params in API mode, Polars operations in local mode.

3. **Insight slugs** — `exodata insights run nearest-stellar-hosts`
   Runs predefined queries. In API mode, hits `/rest/insights/{slug}`. In local
   mode, runs registered SQL against local parquet.

### Agent skill setup

The CLI should be able to install a skill definition that teaches an AI agent
how to use `exodata` to answer questions about exoplanets.

```bash
exodata skill init
```

This writes a skill file (e.g. `SKILL.md` or agent-specific config) that
includes:

- Available commands and examples
- Output format options
- How to interpret results
- Common query patterns

The skill should target agents that already have access to `exodata` on PATH. The
skill file lives in the project's agent config (e.g. Crush `crush.json`, or a
standalone markdown file the user can reference from any agent config).

## Implementation Plan

### Phase 1: Config + backend selection

- Add `dirs` crate dependency for cross-platform config/data directories
- Define config struct and TOML loading (`~/.exodata/config.toml` or
  `$XDG_CONFIG_HOME/exodata/config.toml`)
- Add `--backend` global flag and `EXO_BACKEND` env var
- Add `exodata config` commands (path, get, set)

### Phase 2: API client

- Add `reqwest` dependency for HTTP client
- Implement API client module that hits `/rest/*` endpoints
- Wire existing query/insight commands to use API client when backend is `api`
- Add remote schema commands (`exodata schema stellarhosts`, `exodata schema exoplanets`)

### Phase 3: Download workflow

- Add download commands backed by reqwest (static file URLs from config)
- Progress reporting via `indicatif`
- Verify downloaded files (size check or checksum if available)
- Wire `--data-dir` to default to `~/.exodata/` in local mode

### Phase 4: Unified output model

- Extract shared `OutputFormat` enum (table/json/csv)
- Add `--output` flag to all query commands
- Implement table, JSON, and CSV renderers
- Respect config file default

### Phase 5: Agent skill

- Define skill content (command reference, query patterns, output interpretation)
- `exodata skill init` writes the skill definition to stdout or a file
- Update agent config (AGENTS.md, crush.json) to reference the skill

### Phase 6: REST API enhancements

- Ensure all REST endpoints the CLI needs are in place:
  - `/rest/query?sql=...` (exists)
  - `/rest/stellarhosts`, `/rest/exoplanets` with full filter/sort/page params (exists)
  - `/rest/stellarhosts/schema`, `/rest/exoplanets/schema` (exists)
  - `/rest/insights` — list insights (needs adding)
  - `/rest/insights/{slug}` — run insight (needs adding)
- Static file serving for parquet/metadata via nginx or Axum static routes

## Open Questions

- **Config location**: `~/.exodata/config.toml` (single directory) vs
  `~/.config/exodata/config.toml` + `~/.exodata/` (split config/data)? Single
  directory is simpler; split follows XDG convention. Start with single
  `~/.exodata/` for both config and data unless there's a reason to split.
- **Static file URLs**: should these be under a known path like
  `/static/data/stellarhosts.parquet` or configurable in the CLI config?
  Start with a fixed path relative to the API base URL, add config if needed.
- **Query language scope**: the "invented conf language" for structured queries
  — start with flag-based structured queries (`--sort`, `--filter`, `--columns`)
  rather than a custom DSL. Add a DSL later only if flags become unwieldy.
- **Skill format**: standard markdown (`SKILL.md`) that any agent framework can
  ingest, or framework-specific config? Start with markdown.

## Relevant Specs

- `specs/cli.md` — current CLI command reference
- `specs/cli-client.md` — phased CLI client design (config, backend, download)
- `specs/web-backend.md` — Axum REST API spec
- `specs/data-management.md` — data pipeline
- `specs/ideas.md` — agent/tool access to insights
- `docs/cli.md` — public CLI docs
- `docs/api.md` — public API docs
