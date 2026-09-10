# Current Task Context: Generated System Descriptions (#116)

State: Three-stage pipeline (drafter, editor, critic with correction) plus measurement-bolding, numeric-allowlist, and banned-phrase gates implemented per specs/prose_harness.md; radial_velocity guide entry added; fingerprint v3. Awaiting user-run regeneration of the 15-system baseline.

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
- [x] Serve stored prose on host detail pages: `DescriptionSection` after the
      hero (title stripped, HTML-escaped Markdown), `EXO_CONTENT_DIR` startup
      loader (default `content/systems`, missing content is never an error),
      `get_host_description` server function, shared `system_identifier` moved
      to `exo-core`, Dockerfile ships `content/systems`. Verified: 137 web
      lib tests, split build, SSR smoke test (prose present for described
      systems, absent for others). Deferred: `prepare --all`, prompt embedding,
      ansible upload changes.
- [ ] User commits the current harness, 15-system results, and prose serving.
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
