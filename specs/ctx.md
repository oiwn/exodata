# Current Task Context: Generated System Descriptions (#116)

State: Notes/retry/status round (2026-09-10) implemented: all five review defects fixed, 15/15 baseline regenerated at fingerprint v4, `--failed` retry filter and `descriptions status` proven live, notes.toml manual channel in place, tracked set slimmed to description.md + notes.toml. Critic noise reduced but not eliminated - demote/sample decision pending before the catalog run. Awaiting user review and the git untrack commands below.

Git untrack for the baseline (user-run): `git rm --cached content/systems/*/request.toml content/systems/*/metadata.toml`

## Notes, Retry, and Status Round — 2026-09-10 (implemented)

Fixes the external review's residual defects, adds per-system manual
notes, a failed-only retry filter, a status subcommand, and slims the
tracked file set for catalog scale.

- Gates: articles now ban meta/preparation language ("the guide",
  "supplied", "the request", "the evidence"); tests for each.
- Prepare: `sy_snum > 1` adds a silent constraint attributing stellar
  measurements to the selected host star only (fixed Kepler-16); with
  exactly 3 ranked planets the second-place fact is emitted once, not
  paired (fixed PSR); tests.
- Prompts: critic requires verbatim draft quotes, forbids placeholder
  findings, acknowledges licensed collectives/rankings explicitly;
  drafter/editor forbid mentioning the guide/request/preparation.
- Per-system manual notes: optional tracked
  `content/systems/<id>/notes.toml` (`facts` → publishable_comparisons,
  `guidance` → silent_constraints), merged in preflight; fingerprint
  covers the merge so edited notes regenerate. Never machine-written.
- `generate-batch --failed`: selects systems with `fail.toml` and
  overrides the fingerprint skip (solves the force-trap). Proven live:
  a 12/15 pass with 3 exhausted-retry failures retried cleanly.
- New `dev descriptions status`: per-system state (failed > generated >
  missing), fingerprint version, attempts, token totals, critic findings
  count, timestamps, failure error; aggregate summary line.
- Git layout: `request.toml`/`metadata.toml` now ignored under
  `content/systems/*/`; tracked set is `description.md` + `notes.toml`.
  Baseline 15 untracked via `git rm --cached` (user-run).
- Live verification: two full regenerations; final state 15/15 generated
  in one pass, all five named defects gone (Kepler-16 attribution,
  Kepler-11 count, HR 8799/Proxima meta leaks, PSR double ordinal),
  banned-phrase sweep clean.
- Critic assessment (honest): noise dropped 29 → 9 findings after two
  prompt iterations, but false positives persist (supplied counts,
  licensed rankings, licensed collectives still get flagged). The editor
  demonstrably resists wrong findings and the article floor is gate
  protected, so damage is bounded to wasted critic tokens. Open decision
  for the catalog run: keep as advisory, sample it, or demote to
  recorder-only before spending ~4,735 critic calls.


## Licensed-Facts Round — 2026-09-10 (implemented)

- `prepare.rs` emits curated licensed facts with matching guide entries:
  per-planet `circumbinary = true` (NASA `cb_flag` = 1 only, never inferred
  from `sy_snum`; negative case HD 41004 A), star `host_kind = "pulsar"`
  (any planet method Pulsar Timing), star `age_class = "very young"`
  (age estimate < 0.1 Gyr), and the comparison fact "No orbital period is
  reported for {planet}." when a planet lacks a period.
- Gates: `ValidationContext.licensed_phrases` (built by
  `batch.rs::licensed_phrases`); `banned_patterns` strips licensed phrases
  before matching and bans `circumbinary`, `sun-like`, `about about`
  unconditionally. "pulsar timing" (method label) and "very young"/"pulsar"
  (fact-gated) lift only with their facts; "sun-like" requires a G-type
  supplied spectral label (fixes Kepler-11's leak).
- Pipeline reordered to draft → critic(draft, json mode 768-token cap,
  violations-only strict JSON, 1 parse retry, fail-open) → editor(draft +
  findings + full evidence, gates + preserves_facts vs draft). Editor
  failure names "editor" and preserves the previous description; the
  "critic" failure stage and `corrected` metadata field are gone.
  Fingerprint version 4. DeepSeek alias `deepseek-v4-flash` now serves
  `deepseek-flash` (V4.1 Flash rollout); metadata records the served name.
- Verification: 56 lib tests (licensed-phrase gates, new prepare facts,
  reworked critic tests: notes-reach-editor, editor-failure-preserves,
  fail-open), integration suites, `cargo fmt` clean.
- Live run (15 systems): all generated; ~164k tokens total (~11k/system;
  HR 8799 worst at 27k across two batch passes). Critic findings on 8
  systems, mostly real (Proxima star-vs-system distance, LHS 1140 "quiet",
  HD 189733 aggregation wording); 1 self-cancelling noise entry (K2-18).
  "Sun-like" now absent without a G label, no "about about", no hype, and
  the three prior failures (HR 8799, PSR, OGLE) all generate with licensed
  wording ("very young star", "is a pulsar" + guide decode, no-period fact).
- Residual warts (minor, none blocked): Kepler-16 attributes the single
  supplied stellar mass/radius to "its two stars" and calls a star
  temperature "the system's" (critic missed both); HR 8799 says "the 0.1
  billion years mark that the guide uses" (meta leak); Proxima says
  "supplied spectral type" (prep-language leak); Kepler-11 closes with
  "each of the five with a radius estimate" (six have radii); PSR pairs
  "second-shortest" with "second-longest" year for the middle planet.

## Gate and Critic Round — 2026-09-09 (implemented)

- `validate.rs` gained: `validate_article` chain, measurement-phrase
  bolding gate (number+unit and spectral label must sit inside `strong`),
  numeric allowlist (output tokens ⊆ request-licensed tokens collected by
  `batch.rs::collect_licensed`; raw values/errors never license), banned
  pattern list (position/speed, hype, missing-field commentary,
  raw-uncertainty derivation, unlicensed labels including "pulsar" and
  "hot jupiter"), comma-grouped number handling.
- `batch.rs`: three-stage pipeline - draft, editor, then one critic call
  (512-token cap, strict-JSON findings, one parse retry, fail-open) whose
  findings trigger a single correction pass through the editor gates;
  failed correction fails as "critic" stage preserving the previous
  description. Metadata records critic findings/correction/unparseable.
  `--critic-prompt` flag; fingerprint version 3 covers all three prompts.
- Guide: `radial_velocity` entry (included with RV planets); drafter and
  editor prompts extended with anti-hype and no-new-wording prohibitions.
- `specs/prose_harness.md` corrected (TRAPPIST-1 bolding claim was false;
  editor introduced position wording, drafter introduced 51 Peg hype -
  verified via saved drafts) and extended with implementation notes.
- Verification: 38 lib descriptions tests (three new critic tests:
  correction success, unfixable preservation, unparseable fail-open),
  10 integration, `cargo fmt` clean.

## Two-Stage Round — 2026-09-09 (implemented)

- Batch pipeline: stage 1 drafts with the shared prompt (draft kept as
  gitignored `draft.md`), stage 2 edits with
  `content/stellarhost_editor_prompt.txt` (`--editor-prompt` override)
  receiving draft plus full evidence. Both stages run the Markdown gate;
  the editor additionally requires an identical numeric-token set and
  retained planet names (`preserves_facts`). Three attempts per stage;
  failures name the failed stage; metadata records per-stage attempts and
  token usage with grand totals; fingerprint version 2 covers both prompts.
- Preparation: per-planet `classification` (shared `planet_size_class` in
  `exo-core`, also driving the overview chart labels), `planet_classes`
  guide entry, and mean-density facts (ρ = M/R³, Earth 5.51 g/cm³) for
  estimate `Mass` planets only. Prompt title guidance now encourages a
  short factual descriptive title.
- Prepare UX: "Preparing/Prepared {hostname}" stderr progress with indented
  diagnostic lines; table/CSV show a diagnostics count; JSON keeps the full
  array.
- Web: the article's own H1 renders in the panel (no longer stripped),
  styled slightly larger than body text; softer panel; bold styling for
  generated `strong` spans.
- Verification: 32 lib descriptions tests, 10 integration, web loader tests,
  `cargo fmt` clean.


## Next Batch — Two-Stage Generation and Editorial Quality (locked 2026-09-09)

### Two-stage generation pipeline

- Stage 1 (drafter): the existing shared prompt turns the saved request into
  a complete factual draft. Unchanged.
- Stage 2 (editor): a new `content/stellarhost_editor_prompt.txt` receives
  the draft **plus the full evidence request** (cost is irrelevant) and
  polishes prose: vary sentence openings, remove repeated qualifier words
  ("estimated"/"reported" - "about" is the only hedge a value needs), smooth
  transitions, tighten. It must preserve every number and unit exactly,
  every planet name, every comparison and qualifier meaning, the title, and
  all Markdown rules; add nothing.
- Gates: the existing Markdown gate runs after both stages (3 attempts per
  stage, feedback retry as today). The editor output additionally passes a
  strict fact-preservation gate: numeric-token set equality between draft
  and final after stripping bold markers (no new numbers, none lost - exact,
  because this is a science project) and all planet names still present.
  Violations retry; exhausted retries fail with the previous description
  preserved. Worst case 6 calls per system.
- Artifacts: the draft is kept per system as gitignored
  `content/systems/<id>/draft.md` (same ignore family as `evidence.json`/
  `fail.toml`) and survives successful runs for inspection.
- Metadata records per-stage attempts and token split; the fingerprint
  covers the request plus both prompts plus settings. Batch-only; the manual
  single `generate` command stays single-stage.

### Prepare facts

- Density: for planets with an estimate `Mass`-provenance mass and an
  estimate radius, emit "about N g/cm³ (about Y times Earth's)" from
  ρ = M/R³ with Earth at 5.51 g/cm³. **Skip Msini planets** (their mass is
  a minimum, so density would be a bound); skip bounds and missing inputs.
- Classification: extract the site's existing five radius classes
  (Sub-Earth < 1, Earth-like 1-1.5, Super-Earth 1.5-2.5, Neptune-like
  2.5-4, Jupiter-like > 4 Earth radii, from `exo-core` overview) into one
  shared function used by both the overview chart and preparation. Requests
  carry a per-planet `classification` when the radius is an estimate; a new
  guide entry explains the classes in plain words (single-source precedent,
  like the spectral entry).

### Prepare UX

- Stderr progress mirroring the batch: "Preparing {hostname}" then
  "Prepared {hostname}", each diagnostic as its own indented line.
- The results table shows hostname, paths, and a diagnostics **count**;
  full diagnostic text stays in `--output json`/CSV and on stderr.

### Web

- The article title is part of the generated Markdown and renders inside
  the prose panel (stop stripping the H1 in the loader), styled slightly
  larger than body text - the header is invented by the model, not by site
  chrome. Prompt title guidance moves from "a plain title ... is
  sufficient" toward encouraging a short factual descriptive title while
  keeping the plain form valid.
- Soften the `.host-description` panel (less distinct border/background).

### Verification and rollout

- Tests: editor-stage gates (fact preservation numeric-set equality, name
  presence), two-stage batch flow on the mock server (draft kept, per-stage
  attempts in metadata, stage failure preserves previous description),
  density/classification preparation facts, prepare output shape.
- After implementation: user reruns bulk `prepare --force` +
  `generate-batch` for the 15 systems and reviews pages locally.


## Markdown Quality Round — 2026-09-09

- Shared prompt now demands valid Markdown: single `#` title, paragraphs,
  bolded planet names and measurement values (including spectral type), no
  em dashes or double hyphens (single hyphen allowed), plain sentences
  without appositive re-definitions.
- Deterministic gate (`descriptions/validate.rs`, pulldown-cmark event
  whitelist): one leading H1, paragraphs only, no links/lists/tables/code/
  extra headings, no `—`/`--`, every planet-name occurrence bolded (names
  from the saved request). Violations retry with appended formatting
  feedback, max 3 total attempts; transport failures never retry. Metadata
  records `attempts` and `validation_recovered`; exhausted retries fail with
  the previous description preserved. Usage/elapsed sum across attempts;
  retry variants never touch the stored request or fingerprint.
- `prepare` computes `target_words`: fewer than ten approved comparisons or
  a single planet → 150-300, else 300-600.
- Web: `DescriptionSection` moved after `PlanetsSection` (before
  `ProvenanceSection`).
- Verification: 30 lib descriptions tests (gate unit tests + mock-server
  retry-success and exhausted-retry tests), prepare/batch integration (10),
  137 web lib tests, `cargo fmt` clean. Empty model text remains a transport
  failure (probe rejects it before validation).
- Next: user runs bulk reprepare + generate-batch for the 15 baseline
  systems and reviews pages locally (`cargo leptos watch --split`).

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
- Articles (`description.md`) and manual `notes.toml` live in Git under
  `content/systems/<system-id>/`; `evidence.json`, `fail.toml`,
  `draft.md`, `request.toml`, and `metadata.toml` stay ignored. Input
  refresh is explicit through `prepare`; batch consumes saved requests.
- NASA dates are not a complete change feed (independent stellar-host
  updates and removed rows are invisible to the scan); date-based selection
  is best-effort, explicit regeneration covers the rest.
- Keep the `prose-generation` skill in sync in `.agents` and the ignored
  `.claude` mirror.

## Plan — Full-Catalog Generation

- [x] Compress this context file for the catalog phase.
- [x] Serve stored prose on host detail pages: `DescriptionSection` after the
      hero (title stripped, HTML-escaped Markdown), `EXO_CONTENT_DIR` startup
      loader (default `content/systems`, missing content is never an error),
      `get_host_description` server function, shared `system_identifier` moved
      to `exo-core`, Dockerfile ships `content/systems`. Verified: 137 web
      lib tests, split build, SSR smoke test (prose present for described
      systems, absent for others). Deferred: `prepare --all`, prompt embedding,
      ansible upload changes.
- [ ] User commits the current harness, 15-system results, and prose serving.
- [x] Decide hostname enumeration for bulk prepare: `prepare --all` +
      `--dry-run` implemented (2026-09-10). `Catalog` loads both Parquet
      files once, enumerates hostnames from planet rows, and reports
      per-system results without writing in dry-run mode.
- [x] Dry-run prepare across all hostnames: 4,769 systems, 4,768
      preparable, 1 failure (2MASS J11011926-7732383: no usable
      stellar-host summary row - accept as skip-with-report). Diagnostics
      are benign missing-field notes (mass fields on 3,201 systems,
      radius 1,609, stellar age 1,596, spectype cross-fill 455; median 1
      per system; 833 fully clean; 16 multi-host count notes; 10
      unrecognized mass provenance). 60-second wall time.
- [ ] Confirm batch expectations at scale: ~2,700 tokens/system average
      suggests ~13M tokens and roughly one to two hours at concurrency 4;
      verify rate limits and per-system failure isolation on a slice first.
- [ ] Run catalog-wide generate-batch; spot-check a diverse sample against
      requests; rerun failures; record totals.

## Open Decisions

- Resolved 2026-09-10: missing-field commentary (licensed "No orbital
  period is reported" fact), PSR pulsar decode (`host_kind` fact + method
  label licensing), and age labels (`age_class` fact below 0.1 Gyr);
  resolved same day: Kepler-16 attribution, meta-language leaks, PSR
  double ordinal (prepare fixes + bans).
- Critic role at catalog scale: false-positive rate remains (~9 findings
  on the baseline, mostly licensed restatements); keep advisory, sample,
  or demote to recorder-only before ~4,735 paid critic calls.
- Mixed-provenance ranking (55 Cnc): accept unlicensed, or allow hedged
  uniform-provenance ranking. Current output avoids it.
- Length: outputs run 150-250 words versus the 300-600 target with
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
