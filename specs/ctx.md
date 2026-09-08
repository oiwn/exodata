# Current Task Context: Generated System Descriptions (#116)

State: Quality-improvement phase complete on the 15-system baseline (~12/15
clean). Harness is ready for catalog-wide generation. This file was
compressed for that phase; per-round histories were removed after their
lessons were folded into `specs/cli.md` and the summaries below.

## Goal

Generate a brief natural-language description for each stellar host and its
known planets, for curious general readers, using current NASA data, then
produce descriptions for the full catalog (≈4,735 systems currently `missing`
per `descriptions scan`).

## Current State

### Implemented and verified

- `exodata dev descriptions prepare` — repeated `--hostname` for bulk runs,
  per-system continue-on-error, nonzero exit when any fail. Selection: fullest
  host summary row plus unique planetary default row; the one cross-row
  exception is `st_spectype` (filled from any host row, with a diagnostic).
  Paired evidence/request writes with ordinary-failure rollback.
- `exodata dev descriptions generate-batch` — concurrency 4, model
  `deepseek-v4-flash`, thinking/streaming off, 60-second timeout, no retries.
  SHA-256 fingerprints (prompt + canonical request + settings) skip unchanged
  inputs — demonstrated live: a changed-fact reprepare regenerated exactly the
  four affected systems and skipped eleven. Paired description/metadata
  installs; latest failure in ignored `fail.toml`; pending staging files block
  another operation.
- Deterministic fact generation (`comparisons()` in `prepare.rs`): stellar
  mass/radius/temperature versus the Sun; planet radius/mass versus Earth;
  Jupiter-scale context when a mass clearly exceeds 317.8 Earth masses; year
  versus Earth; first- and second-place extrema ranked by point value with
  rounded-tie suppression, estimates-only filtering, and uniform provenance
  for mass (`Mass`/`Msini` wordings). Baseline comparisons keep interval
  separation. Contract: [cli.md](cli.md#stellar-host-input-preparation).
- Shared prompt `content/stellarhost_prompt.txt`: conversion-not-analysis
  framing, exhaustive comparison list, anti-position and anti-commentary
  examples. Guide `content/prompts/stellarhost_guide.toml` is the single
  source of explanatory truth (spectral decode, M-sin-i inclination, units,
  Jupiter mass, limits, transit, year); it is compiled in — rebuild and
  reprepare after guide edits. Irrelevant entries are excluded automatically.
- Fixed baseline (15 systems, do not silently replace): 51 Peg, HD 41004 A,
  Kepler-11, LHS 1140, TRAPPIST-1, HR 8799, PSR B1257+12, Kepler-16, 55 Cnc,
  Proxima Cen, HD 189733, GJ 1214, K2-18, HD 10180, OGLE-2016-BLG-1195L.
- Verification baseline: 23 lib + 10 description integration tests pass;
  `cargo fmt` clean. No Clippy/workspace/hydration claims beyond that.

### Quality status (latest full review)

- Clean: 51 Peg, LHS 1140, Kepler-11, Kepler-16, HD 10180, Proxima Cen,
  GJ 1214, K2-18, PSR B1257+12 (rankings), TRAPPIST-1 (one vague referent),
  HD 189733 (mild "Sun-like" stretched to `K2 V`).
- Remaining violations: missing-field commentary (HR 8799 e period; OGLE
  period and "no additional planets"); 55 Cnc "innermost system of planets"
  plus "most massive of the set" (mixed-provenance sets are unranked by
  design); PSR "is a pulsar" decoded from the supplied method name.
- Driving lesson across all rounds: precomputed facts crowd out derivations.
  Every licensed fact class (spectral decode, M-sin-i mechanics, mass
  rankings, ordinals) stopped being violated. The model fills gaps in the
  fact list, so fix gaps in facts rather than adding prose prohibitions.

### Agreed constraints (unchanged)

- No trial directories, Python orchestration, editorial states, or root
  `tmp/` workflow. Evidence: current NASA dataset plus the curated guide; no
  per-system web research. Metadata never stores credentials.
- Articles and requests live in Git under `content/systems/<system-id>/`;
  `evidence.json` and `fail.toml` stay ignored. Input refresh is explicit
  through `prepare`; batch consumes saved requests.
- NASA dates are not a complete change feed (independent stellar-host
  updates and removed rows are invisible to the scan); date-based selection
  is best-effort, explicit regeneration covers the rest.
- Keep the `prose-generation` skill in sync in `.agents` and the ignored
  `.claude` mirror.

## Plan — Full-Catalog Generation

- [x] Compress this context file for the catalog phase.
- [ ] User commits the current harness and 15-system results.
- [ ] Decide hostname enumeration for bulk prepare: `prepare --all`, a
      scan-driven selection, or a generated hostname list (prepare currently
      requires explicit `--hostname` arguments).
- [ ] Resolve or defer the open decisions below; they will propagate to
      ~4,735 articles if left as-is.
- [ ] Dry-run prepare across all hostnames; collect and classify failures
      (ambiguous selection, identifier collisions, unusable host rows) and
      agree on skip-with-report handling.
- [ ] Confirm batch expectations at scale: ~2,700 tokens/system average
      suggests ~13M tokens and roughly one to two hours at concurrency 4;
      verify rate limits and per-system failure isolation on a slice first.
- [ ] Run catalog-wide generate-batch; spot-check a diverse sample against
      requests; rerun failures; record totals.
- [ ] Serve stored descriptions on host detail pages (application
      integration to be specified in web specs).

## Open Decisions

- Missing-field commentary (HR 8799 e, OGLE): accept, or license an explicit
  "no supplied period" fact.
- PSR-style decode of the "Pulsar Timing" method name into "a pulsar":
  accept as licensed decode (spectral precedent) or forbid.
- Mixed-provenance ranking (55 Cnc): accept unlicensed, or allow hedged
  uniform-provenance ranking.
- Length: outputs run 150–250 words versus the 300–600 target with
  permission to be shorter; decide whether to lower the target.
- Serving integration, deployment packaging, and initial language coverage
  remain unspecified.

## Reference

- Contracts: [cli.md](cli.md) (prepare, batch, fingerprinting, scan),
  [testing.md](testing.md). Data: [data-management.md](data-management.md).
- Workflows and review criteria: `prose-generation` skill.
- DeepSeek API: https://api-docs.deepseek.com/ (model aliases can change
  behind a stable name; metadata records the returned model).
- Issue: https://github.com/oiwn/exodata/issues/116
