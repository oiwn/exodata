---
name: prose-generation
description: Prepare stellar-host prose evidence and requests in this Exodata repository, and run or review authorized generation experiments. Use for description input preparation and prose evaluation, not general catalog queries or publishing.
---

# Stellar-host prose generation

Run from the repository root. Read [the CLI contract](../../../specs/cli.md#stellar-host-input-preparation)
and [current task context](../../../specs/ctx.md) before experiments. Technical
contracts live in specs; do not duplicate data selection or conversion logic in
ad hoc scripts.

## Prepare and inspect

The fixed baseline is 51 Peg, HD 41004 A, Kepler-11, LHS 1140, TRAPPIST-1,
HR 8799, PSR B1257+12, Kepler-16, 55 Cnc, Proxima Cen, HD 189733, GJ 1214,
K2-18, HD 10180, and OGLE-2016-BLG-1195L.
Keep it fixed while improving results; do not silently replace a difficult case.

```sh
cargo run --locked -p exodata -- dev descriptions prepare --hostname "LHS 1140"
```

This reads current local Parquet data and writes `evidence.json` and
`request.toml` under `content/systems/lhs-1140/`. It makes no model calls.
Use `--output-dir` for alternate experiments. Use `--force` only when replacing
the current preparation is intended; it preserves articles and generation
metadata, but overwrites edits to the preparation files.
`evidence.json` is ignored generated data; requests and shared prompts remain
tracked. Refresh evidence and request together using `prepare`, never separately.
The command stages both files and rolls back ordinary installation failures.
If it reports failed rollback or an existing `.prepare` directory, inspect the
reported backups before recovery; do not discard them or blindly retry.

Inspect evidence selection, missing measurements, uncertainty/limit flags, and
mass provenance before evaluating prose. Check the exact-host planet count
separately from the catalog system count. A missing unique default row is an
input problem to resolve, not permission to fill from another row. Host
spectral type is the one cross-row exception: when the selected summary row
lacks `st_spectype`, `prepare` fills it from any host row and records a
diagnostic.

The single shared system prompt is
[content/stellarhost_prompt.txt](../../../content/stellarhost_prompt.txt).
Per-system editorial notes go in `request.toml`'s `silent_constraints`; they
must never become reader-facing prose. Shared explanatory text lives in
[the guide](../../../content/prompts/stellarhost_guide.toml), compiled into
preparation; rebuild/reprepare after editing the guide. Prompt edits are read
directly by `generate` and do not require a rebuild.

## Generate and regenerate

When generation is requested, check only whether `DEEPSEEK_API_KEY` is
present/nonblank; never print it. Use the native Rust command:

```sh
cargo run --locked -p exodata -- dev descriptions generate-batch \
  --input-dir content/systems --concurrency 4 --output json
```

Optional repeated `--hostname` arguments limit the batch. The command consumes
saved requests, preserving their edits. Run `prepare` explicitly after source
data or preparation-guide changes. Shared-prompt changes are picked up directly.

The default cap is 1,536 output tokens per call, thinking disabled, no retries,
and a 60-second timeout. The word target remains 300–600 with permission to be
shorter. Keep the fixed five-system set during harness improvement; do not add
repair calls or expand a requested baseline run to the full catalog.

An unchanged successful fingerprint is skipped. `--force` requests a new result
with unchanged inputs. Fingerprints compare the prompt, canonical structured
request, and generation settings; they do not inspect live Parquet data.

Everything per-system lives in `content/systems/<system-id>/`: tracked
`request.toml`, `description.md`, and compact `metadata.toml`; ignored
`evidence.json` and latest `fail.toml`. Success updates prose and metadata
together and clears old failure information. Failure keeps the previous success
and records diagnostics and any available returned text/usage in `fail.toml`.
Do not build Python orchestration, trial folders, or a root `tmp/` workflow.
Do not copy the shared prompt per system or create permanent response archives.

Progress and aggregate usage go to stderr; stdout contains table/JSON/CSV
outcomes. Account/rate-limit/storage failures stop new scheduling while in-flight
requests finish. A nonzero exit may include successful systems; inspect outcomes
before rerunning. Pending `.generate` or `.fail-write` files require inspection
before recovery, just like `.prepare` artifacts.

## Check generation quality during development

Compare prose with supplied facts: numbers, bounds, Mass versus Msini, radius
wording, exact-host counts, and approved comparisons. Check for unlicensed
classifications/history (a supplied spectral type restated through the guide's
`spectral` entry is licensed), orbital-position/speed inferences, unapproved
mass rankings, repeated inventories, and leaked diagnostic language. A normal
model response is not proof of factual correctness. Do not invent returned
usage or timing.

Report findings in task context while improving the harness. There are no
draft/accepted/published states or per-article approval workflow. Never stage or
commit unless explicitly requested. Do not start a web server or download data
for offline preparation/generation work.
