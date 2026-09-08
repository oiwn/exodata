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

The fixed baseline is LHS 1140, Kepler-11, TRAPPIST-1, 51 Peg, and HD 41004 A.
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
input problem to resolve, not permission to fill from another row.

The single shared system prompt is
[content/stellarhost_prompt.txt](../../../content/stellarhost_prompt.txt).
Per-system editorial notes go in `request.toml`'s `silent_constraints`; they
must never become reader-facing prose. Shared explanatory text lives in
[the guide](../../../content/prompts/stellarhost_guide.toml), compiled into
preparation; rebuild/reprepare after editing the guide. Prompt edits are read
directly by `generate` and do not require a rebuild.

## Authorized generation trials

Input preparation alone does not authorize paid API trials. When trials are
requested, check only whether `DEEPSEEK_API_KEY` is present/nonblank; never print
it. Do not start a web server or download data.

```sh
cargo run --locked -p exodata -- dev descriptions generate \
  --input content/systems/lhs-1140/request.toml \
  --system-prompt content/stellarhost_prompt.txt \
  --max-tokens 1536 --output json
```

Use thinking disabled, no retries, and the existing 60-second timeout. The
request target remains 300–600 words with permission to be shorter. Do not
change the token cap, add repair calls, or expand to catalog-wide runs without
an explicit change in scope.

Before a trial, copy its exact request, evidence, and shared prompt into a new
named trial directory under `tmp/`; pass the copied request and prompt to the
generator. Save the complete JSON report, not just extracted article text.
Record command/settings and review alongside it. Preserve previous trials and
the existing manually reviewed baselines. On failure, record the failure; do
not invent missing response text, usage, timing, or returned model metadata.
The current CLI discards non-normal responses, so failed calls may have no
response artifact. Automatic artifact capture is not implemented yet.

## Review

Separate deterministic input verification from prose review. Check every
factual claim against publishable input, including units, rounding, upper/lower
limits, and minimum-mass quantities. Scientific plausibility does not make an
unapproved claim supported. Look for newly calculated ratios, mass rankings,
orbital-speed/spacing inferences, remembered classifications, and leaked notes.

Then assess repetition, unnecessary numerical inventories, hype, filler,
readability, and useful coverage. Record factual and editorial failures
separately; avoid exact-prose assertions or treating a model critique as proof
of correctness. Store actual usage and timing only when returned.

These are preparation and review artifacts, not published articles. Do not
advance generation source dates, overwrite manual articles, stage, or commit
as part of an experiment unless specifically requested.
