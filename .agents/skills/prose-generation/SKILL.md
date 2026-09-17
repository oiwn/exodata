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

The drafter prompt is
[content/stellarhost_prompt.txt](../../../content/stellarhost_prompt.txt) and
the optional repair prompt is
[content/stellarhost_repair_prompt.txt](../../../content/stellarhost_repair_prompt.txt);
both are read at batch runtime (`--system-prompt`/`--repair-prompt` override,
with `--style-prompt` retained as a legacy CLI alias).
Per-system editorial notes go in `request.toml`'s `silent_constraints`; they
must never become reader-facing prose. Shared explanatory text lives in
[the guide](../../../content/stellarhost_guide.toml), compiled into
preparation; rebuild/reprepare after editing the guide.

## Analyze generated prose

`dev descriptions analyze` reads a content directory without model calls
and is the evaluation instrument for comparing generations:

```sh
cargo run --locked -p exodata -- dev descriptions analyze summary
cargo run --locked -p exodata -- dev descriptions analyze ngrams --openers --min-n 2 --max-n 3
cargo run --locked -p exodata -- dev descriptions analyze templates --min-count 20
cargo run --locked -p exodata -- dev descriptions analyze tropes
cargo run --locked -p exodata -- dev descriptions analyze anomalies --z-threshold 3
cargo run --locked -p exodata -- dev descriptions analyze report --output-path report.md
```

`summary` gives corpus length/TTR distributions; `ngrams --openers`,
`templates` (number/unit-masked sentences), and `tropes` expose
repeated phrasing; `metadata` aggregates tokens and attempts;
`anomalies` flags outliers and cross-system duplicate sentences. All
support `--output json/csv/table`. `snapshot --output-path
content/stats/<name>.json` records durable machine-readable stats;
`compare --baseline-dir` diffs two corpora.

## Experiment on the fixed 20-system set

Prompt, gate, and model changes are evaluated without touching served
prose:

```sh
cargo run --locked -p exodata -- dev descriptions experiment --label v2
cargo run --locked -p exodata -- dev descriptions analyze --label v2 \
  compare --baseline-dir content/systems
```

The set is the 15-system baseline plus Kepler-42, L 98-59, TOI-178,
V1298 Tau, and HD 260655. Requests are always refreshed first, so the
experiment runs on current preparation semantics. Outputs are
`description_v2.md` and friends; `description.md` is never modified.
Use `--force` to regenerate despite matching variant fingerprints.
Retry with `generate-batch --label v2 --failed`. Analysis reads the matching
failure file; compare excludes failed latest attempts even when an older
article survives, and reports both failure counts and successful pairs.

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

Generation is one writing pass plus at most one validation repair (fingerprint
v10). Compact JSON is projected after notes merging, preserving reader-facing
facts and private guidance while omitting audit values/errors and source
identifiers. Ordinary mass and minimum mass have distinct keys without raw
audit notation. Model-facing facts and normalized output omit measurement
`about` prefixes, preserving values, units, bounds, and minimum masses. Do not
add an approximation note or disclaimer. Saved preparation remains unchanged.

Normalize and validate the first response; install it directly if valid. Only a
concrete validation failure triggers one repair with the failed article, specific
violations, and compact facts. Preserve unaffected supported facts while allowing
invalid values to be corrected. No always-run editor or numeric-set lock against
a defective draft remains. Transport, malformed/empty, and truncated responses
are not automatically retried. Keep the normalized first response in the draft
artifact even when invalid; preserve previous successes on failure.

Every requested planet must appear. Omit Earth-year comparisons and conversions;
state orbital periods in days. Preparation no longer emits these comparisons,
and generation filters them from older requests and merged notes. Approved
period rankings among the listed planets remain available. Reject
explicit system-wide host-star measurements and the observed stellar minimum
mass error. Prompts require natural count language, explicitly licensed
significance and mass rankings, and no mixed ordinary/minimum-mass ranking;
they also request a grammar, tautology, and repeated-frame pass. Existing
structural, numeric, emphasis, and policy checks remain; repetition analysis is
advisory. Review units, bounds, associations, provenance, and editorial quality
manually: these checks are not complete verification.

Usage includes every attempted stage on success and failure, with
`usage_complete = false` when usage is missing. Analysis/status prefer latest
matching failures and recover historical totals from stage records without
rewriting artifacts. New stages are `draft` and optional `repair`; historical
`style` records remain readable.

Keep the current model, 1,536 output-token cap, disabled thinking, 60-second
timeout, and soft length targets. Run the fixed 20-system experiment under a
fresh label and target 20 valid articles within 100,000 reported tokens (60–100k
is acceptable and lower usage is welcome), including failures
and repairs. Diagnose a missed benchmark before another paid run.

Unchanged successful fingerprints are skipped; `--force` regenerates. Version 10
covers the notes-merged request, writer and repair prompts, and settings, not
live Parquet data.

Everything per-system lives in `content/systems/<system-id>/`, all
untracked private local data: `description.md` and optional hand-edited
`notes.toml` (`facts` merge into publishable comparisons, `guidance`
into silent constraints; the fingerprint covers the merge, so edited
notes regenerate); `request.toml`, `metadata.toml`, `evidence.json`,
`draft.md`, and latest `fail.toml` are generated artifacts. Success
updates prose and metadata together and clears old
failure information. Failure keeps the previous success and records
diagnostics and any available returned text/usage in `fail.toml`.
Retry failed systems with `generate-batch --failed` (selects `fail.toml`
systems and overrides the fingerprint skip); add `--label L` to select and
clear only `fail_L.toml`.
`--failed` with no hostname filter and no matching failures returns
`No failed systems` successfully, with empty structured results and no credential
lookup, calls, or writes; missing preparation remains an error.
Inspect served catalog state with
`dev descriptions status` (per-system state, usage totals, failure
errors, aggregate summary). Bolding and the density possessive are
algorithmic (`normalize_article` runs before the gates; the gate is a
backstop), and `dev descriptions normalize` re-applies the pass to
stored files. Use `normalize --label pass2` for only the selected variant.
This deterministic cleanup rewrites changed articles without touching metadata,
requests, drafts, or other variants and makes no model calls. Ordinary uses of
`about` (for example, `about its composition`) remain. These per-system commands print compact single-line rows
by default; use `--output json`/`csv`/`table` otherwise. Do not build
Python orchestration, trial folders, or a root `tmp/` workflow.
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
