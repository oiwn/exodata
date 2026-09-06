# Current Task Context: Specify Generated System Descriptions (#116)

State: specification in progress.

## Goal

Generate a brief natural-language description for each stellar host and its known planets, for curious general readers, using current NASA data. The earlier 300–600-word target is provisional given the request for short/brief descriptions; settle the length before implementation.

## Plan

- [ ] Finalize Phase 1 metadata storage and scan-report defaults for date-based regeneration selection.
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
| `missing` | No generated description or corresponding metadata exists. |
| `outdated` | Current system source date is later than the recorded source date. |
| `current` | Current and recorded source dates match. |
| `unknown` | Required dates are unavailable, or the current source date is earlier than the recorded date. |

`missing` takes precedence when generation artifacts are absent. When an
individual record lacks `rowupdate`, its available `releasedate` still
participates in the system maximum.

### Command and Report

Proposed command surface:

```bash
exodata dev descriptions scan
exodata dev descriptions scan --output json
```

Report hostname, status, recorded source date, and current source date. Reuse
the CLI's output conventions. Decide whether the default report includes all
systems or only `missing`, `outdated`, and `unknown` systems.

This is date-based selection, not a dataset diff. Same-date corrections,
removed records, and independent `stellarhosts` updates are not automatically
detected; explicit regeneration handles those cases. No snapshots, previous
data values, or download versioning are required.

### Verification and Remaining Decisions

- Verify grouping across multiple planets and reference rows; either date
  field may supply the system maximum.
- Verify each status, missing update dates with available release dates,
  systems without usable dates, and dates moving backwards.
- Verify the scan is read-only and text/JSON reports describe the same results.
- Finalize metadata fields, system identifier encoding/collision handling,
  default report filtering, and handling of malformed dates or invalid metadata
  before implementation.

## Open Decisions

- Phase 1: metadata fields, system identifier encoding/collision handling, default report filtering, and invalid-input handling.
- Input field selection, reference/default-row policy, optional precalculations, and final target length for brief descriptions.
- Generation stage count, prompt/output contracts, validation criteria, repair limits, and pilot acceptance criteria.
- Request TOML schema and multistep artifact layout, review/publish states, handling of removed systems, and behavior when published articles become stale.
- Initial language coverage and how translations depend on source articles and evidence.
- CLI command surface, concurrency defaults, timeouts, retry policy, run limits, and usage reporting.
- Article placement on host detail pages and how stored content is packaged with deployment.

## Next

Finalize the remaining Phase 1 choices, then define the brief-description input and generation contract. Operate on current files; download versioning and diffs are excluded. Keep remaining proposals distinct from agreed decisions.
