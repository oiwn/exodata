# Current Task Context: Specify Generated System Descriptions (#116)

State: Phase 1 implemented and verified; generation specification remains active.

## Goal

Generate a brief natural-language description for each stellar host and its known planets, for curious general readers, using current NASA data. The earlier 300–600-word target is provisional given the request for short/brief descriptions; settle the length before implementation.

## Plan

- [ ] Implement shared source-row selection: fullest stellar-host summary row
  and unique planetary default row; preserve source values and missing fields.
- [ ] Align detail displays/exports and verify selection before resuming prompt work.

- [x] Finalize Phase 1 metadata storage and scan-report defaults for date-based regeneration selection.
- [x] Implement and verify the read-only Phase 1 scanner.
- [ ] Define generation inputs from current host/planet records and any precalculated values.
- [ ] Define the narrative guide, generation stages, validation, and editorial workflow.
- [ ] Specify CLI operations, resumable batch execution, article storage, and serving behavior.
- [ ] Resolve open decisions and consolidate the agreed technical contract into dedicated specs before implementation.

## Agreed Scope

- Issue: https://github.com/oiwn/exodata/issues/116 — generate brief system descriptions for host detail pages.
- Anomaly discovery or highlighting distinctive features is not a required stage.
- Evidence sources: current NASA dataset, a curated explanatory guide, and optionally precalculated values extracted from that dataset; no per-system web research initially.
- Use DeepSeek Flash through `reqwest`; multistep generation remains under consideration. Store the exact prompts and generation metadata as artifacts alongside generated output.
- Generation metadata includes generation time, model information, and request settings, excluding credentials.
- Generated articles may live in Git and must support selective editing and regeneration through development tooling.
- Operate on current data and use available NASA `releasedate` and `rowupdate` values to select systems for regeneration. Dataset diffs, previous data values, snapshots, and download versioning are out of scope.

## Findings

- Data sources are `ps` and `stellarhosts`; the Justfile and local data are aligned. Source semantics, date fields, and inspection commands are documented in [data-management.md](data-management.md).
- Local VOTable and Parquet inspection confirmed release/update fields are present. `rowupdate` can be missing even when `releasedate` is populated; regeneration selection must handle this.
- NASA dates are not a complete change feed: independently updated stellar-host data has no equivalent timestamp, and removed planetary rows cannot be discovered with a date filter alone. Decide the acceptable regeneration behavior without introducing dataset history.
- Host and planet canonical summaries already expose adopted values, measurement ranges, disagreements, and some provenance. Relevant computation currently lives in web server modules and would need sharing with offline tooling.
- Existing summary flags are not anomaly classifications: numeric `disputed` means multiple distinct values, not significant disagreement. Evidence preparation must preserve relevant uncertainty, limit, and provenance information beyond the current summary payloads.
- DeepSeek documents `deepseek-v4-flash`. JSON mode requires an explicit JSON instruction and example; empty content and truncated output remain possible. Validate structure and evidence independently of JSON parsing.
- Model aliases may change behind a stable name; record available model/version information and generation settings rather than assuming reproducibility from the alias alone.
- References: https://api-docs.deepseek.com/ and https://api-docs.deepseek.com/guides/json_mode/

## Proposed Direction — Not Yet Finalized

- **Regeneration:** use the Phase 1 scan to select systems, then generate from their current data. Also support explicit system selection.
- **Evidence:** prepare structured inputs from current host and planet records, summaries, relevant measurements, flags, references, and chosen precalculations. Preserve enough provenance to validate generated statements.
- **Narrative control:** load a repository-owned guide defining audience, terminology, treatment of missing/conflicting evidence, permitted comparisons, and writing examples. Keep technical specs focused on interfaces and behavior; explanatory content belongs in the guide.
- **Generation:** input preparation → description generation → validation → review. Additional outline or critique calls are optional pending evaluation. Validate structure and supported numeric claims; model critique alone cannot establish factual correctness.
- **Editorial storage:** Git-tracked Markdown descriptions with exact prompts and generation metadata. Define how regeneration preserves manual edits and produces reviewable replacements. Keep existing descriptions intact when replacement generation fails.
- **Batch tooling:** offline Tokio async tasks with bounded in-flight API calls via a semaphore, bounded retries, checkpoints, and resumable execution. Begin with a varied pilot of roughly 20 systems to assess quality, latency, and token usage before catalog-wide generation.
- **Serving:** serve reviewed stored articles through the application without model calls during page requests.

## Phase 1: Date-Based Regeneration Scan

Implement a read-only command under the existing `exodata` CLI development
commands (`crates/exo-cli`). It scans the current local exoplanets Parquet
dataset and reports which systems need descriptions or may need regeneration.
It does not call the model, modify generation artifacts, or download data.

### Metadata

Store each system's files together under `content/systems/<system-id>/`,
tracked in Git:

```text
content/systems/hd-189733/
├── description.md
├── metadata.toml
└── request.toml
```

- `description.md`: generated Markdown description displayed to visitors.
- `metadata.toml`: exact NASA hostname, source date, generation timestamp,
  model information, and generation settings; never credentials.
- `request.toml`: structured generation input containing writing instructions,
  host data, planet records, and any precalculated values supplied to the model.
  Retain the input actually used for generation as an artifact.

Use a filesystem-safe system identifier for the directory and preserve the
exact NASA hostname in metadata. Identifier encoding and collision handling
remain to be specified. Per-system files support selective editing/regeneration
and independent writes by concurrent generation jobs.

The CLI sends the serialized TOML input as message content through `reqwest`.
The DeepSeek HTTP API envelope remains JSON; `request.toml` describes the
model's input, not the wire format of the HTTP request. Exact TOML fields and
artifact layout for multiple generation steps remain open. Phase 1 only reads
existing artifacts for reporting; creating requests and descriptions belongs
to the generation phase.

Record the system's `hostname`, source date used for successful generation,
and a separate `generated_at` timestamp. Compare source dates, not NASA dates
against local generation time: newly downloaded records may have release dates
earlier than the generation run.

Generation saves the source date from the data it actually used and updates
metadata only after successfully saving the description. Failed generation
must not advance the recorded source date.

### Selection Rules

1. Group current `ps` records by `hostname`.
2. For each system, take the latest available date across both `rowupdate` and
   `releasedate`, across all its planet records. Do not restrict this scan to
   `default_flag = 1`.
3. Compare that date with the source date recorded in the system's generation
   metadata.

| Status | Meaning |
| --- | --- |
| `missing` | Description or metadata is absent, or description is empty/whitespace-only. |
| `outdated` | Current system source date is later than the recorded source date. |
| `current` | Current and recorded source dates match. |
| `unknown` | Dates are unavailable or malformed, metadata is invalid/ambiguous, artifacts cannot be read, or the current source date is earlier. |

Invalid input takes precedence, then `missing` when artifacts are absent. When an
individual record lacks `rowupdate`, its available `releasedate` still
participates in the system maximum.

### Command and Report

Agreed command surface:

```bash
exodata dev descriptions scan
exodata dev descriptions scan --all
exodata dev descriptions scan --output json
exodata dev descriptions scan --data-dir data --content-dir content/systems
```

Report hostname, status, recorded/current source dates, metadata path, and reason
through shared table/JSON/CSV output. Default to systems needing attention;
`--all` includes current systems. Match exact metadata hostnames, never directory
names. See [cli.md](cli.md#description-regeneration-scan) for the full contract,
including malformed metadata diagnostics, failure behavior, and ordering.

This is date-based selection, not a dataset diff. Same-date corrections,
removed records, and independent `stellarhosts` updates are not automatically
detected; explicit regeneration handles those cases. No snapshots, previous
data values, or download versioning are required.

### Verification and Remaining Decisions

- Source columns accept validated `YYYY-MM-DD HH:MM:SS` timestamps as well as
  dates, comparing only the calendar date. Metadata dates remain strict.
  The corrected local scan reports 4,735 missing systems and no unknown rows.
- Local Parquet scan completed successfully. Six synthetic-data integration
  tests pass, including CLI formats/defaults and read-only checks. Ran
  `cargo fmt --all`; Clippy with warnings denied passes for the library, binary,
  and scanner tests. The all-targets Clippy check encounters existing warnings
  in `examples/create_fixtures.rs`.
- Verify grouping across multiple planets and reference rows; either date
  field may supply the system maximum.
- Verify each status, missing update dates with available release dates,
  systems without usable dates, and dates moving backwards.
- Verify the scan is read-only and text/JSON reports describe the same results.
- Parse hostname and optional source_date, tolerating other metadata fields.
  Directory identifier encoding/collision handling belongs to generation.

## Open Decisions

- Generation: system identifier encoding/collision handling and detailed metadata/request schemas.
- Final input schema and field coverage, optional precalculations, and length defaults; current experiments use 300–600 words per request with permission to be shorter. Source selection is now agreed: fullest host summary row and unique planetary default row, with no cross-row filling.
- Generation stage count, prompt/output contracts, validation criteria, repair limits, and pilot acceptance criteria.
- Request TOML schema and multistep artifact layout, review/publish states, handling of removed systems, and behavior when published articles become stale.
- Initial language coverage and how translations depend on source articles and evidence.
- CLI command surface, concurrency defaults, timeouts, retry policy, run limits, and usage reporting.
- Article placement on host detail pages and how stored content is packaged with deployment.

## DeepSeek Probe

- Added `exodata dev descriptions probe` before generation input design;
  see [cli.md](cli.md#deepseek-connectivity-probe).
- Use `DEEPSEEK_API_KEY` from the environment, one fixed request, at most 32
  output tokens, thinking disabled, a 60-second timeout, and no retries.
- Developer's live probe passed: `OK`, 9 prompt tokens, 1 completion token,
  normal stop, 743 ms.
- Added `descriptions generate --input <file> --max-tokens 256` for manual
  experiments with arbitrary text/TOML; it sends the file verbatim and uses
  the same single-request settings and report as the probe. Live generation
  tests so far were run by the developer; structured generation inputs remain
  unspecified. Next session the user intends to expose the key to the agent
  for bounded API evaluations.

## Next

### Baseline and Preparation Handoff (2026-09-08)

- Keep these five systems fixed while improving generation quality: LHS 1140,
  Kepler-11, TRAPPIST-1, 51 Peg, and HD 41004 A. Inspect their current selected
  rows when implementing automatic input preparation; retain the existing
  manual experiment files as baselines.
- Implement automatic evidence/input preparation under
  `exodata dev descriptions` (proposed subcommand: `prepare`). Default
  `--output-dir` to `content/systems`, placing each system's artifacts in its
  own directory. Normalize system directory names by removing escape/unsafe
  characters and replacing spaces with dashes; preserve the exact NASA
  hostname in artifacts. Specify exact normalization and collision handling
  before implementation.
- Add a repository `prose-generation` skill explaining how future LLM agents
  prepare inputs, run bounded generation trials, preserve artifacts, and
  evaluate factual support and prose quality.
- First fix `cargo test`, then pause for the user's commit before implementing
  preparation or the skill. Fixed the host export/cache test to use Tokio's
  multithreaded runtime required by Polars. Ran `cargo fmt --all` and
  `cargo test`: 134 tests passed, no failures. This does not establish workspace
  or hydration verification.

Current manual experiment: `generate --system-prompt <file>` separates editorial
rules from evidence. Kepler-11 v2 in `tmp/` uses prose-ready values, approved
comparisons, explicit upper-limit qualifiers, and uncertainty cautions instead
of raw error/limit columns. Raw evidence remains available locally. This is an
experiment, not the final input schema; the developer has run revision 3.

Kepler-11 manual review: the compact input preserved the mass upper limit and
removed the earlier mass contradiction, but output still inferred orbital
spacing/observing feasibility, calculated an unapproved ratio, and exposed an
editorial caution. Prompt revision 3 updates the same v2 input and system file:
separate publishable comparisons from silent constraints and require a specific
supported ending. Measurements and the 300–600-word target are unchanged.

### Prompt Evaluation Handoff (2026-09-08)

- Latest Kepler-11 revision preserved the upper limit and removed earlier
  numerical contradictions and unsupported composition claims. Remaining
  issues: repeated periods/radii, filler, catalog-style prose, and describing
  all six masses as estimates despite one being an upper limit.
- LHS 1140 was tested next: one star, two planets. Host input uses the unique
  fullest summary row (Cadieux et al. 2024, 10/10 fields); each planet uses its
  default `ps` row. Age is a lower limit, not an exact age.
- Preserved local experiment files: `tmp/description-system.txt`,
  `tmp/request-lhs-1140.toml`, `tmp/response-lhs-1140.md`, and
  `tmp/evaluation-lhs-1140.toml`. Response wording comes from the user's pasted
  output, with terminal wraps removed. Subsequent revisions should use new
  filenames to preserve this baseline. These are not published articles.
- LHS 1140 preserved core numbers and the age qualifier but added classification,
  an unapproved mass ratio/ranking, orbital-speed wording inferred from period,
  and extra transit wording. Repetition/hype remain. See the saved evaluation.
  Returned usage, timing, finish reason, and model metadata were not pasted;
  do not invent them.
- Prompt structure: a reusable system message and TOML user message separating
  publishable facts/comparisons, explanatory guide, audit metadata, and silent
  constraints. Pass explicit lower/upper-limit qualifiers and relevant uncertainty
  cautions; retain raw errors/flags in source evidence. Precalculate and approve
  conversions/comparisons. Prompt compliance remains imperfect despite these rules.
- Next session: check only whether `DEEPSEEK_API_KEY` is present, never print it.
  The user uses fish and will launch the agent from the shell with the exported
  variable. Develop the small evaluation-harness contract and perform bounded
  live comparisons, capturing exact system/user input, response, request settings,
  returned metadata/usage/timing, and review results. Separate deterministic
  evidence tests from live prose evaluation; no exact-prose assertions,
  catalog-wide batch, or automatic repair loop yet.
- Retain the 1,536-output-token cap, thinking disabled, and no retries for article
  trials unless explicitly changed. The agent has not made generation calls yet.
  No server is needed: the user handles manual web testing. Do not start a server
  to validate offline generation work.
- Source-selection implementation was added. Shared selector and canonical tests
  passed; export/related-planet tests added later and hydration verification were
  not run before the user stopped server testing. The user subsequently reported
  it looked good. Do not claim remaining checks passed or resume server work as
  part of article evaluation.

Define the brief-description input and generation contract. Operate on current files; download versioning and diffs are excluded. Keep remaining proposals distinct from agreed decisions.
