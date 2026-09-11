# Prose Generation Harness

## Purpose

This specification records the editorial quality requirements for generated
stellar-host descriptions. It supplements the CLI contract: source requests
remain the sole authority for reader-facing claims, and prose must not turn
measurements or request ordering into new scientific conclusions.

## Baseline Editorial Audit

The fixed 15-system baseline was reviewed against its saved `request.toml`
files and the shared drafter/editor prompts. The audit table below refers to
the earlier two-stage, fingerprint-version-2 outputs dated 2026-09-09; the
current artifacts are draft-critic-edit, fingerprint-version-4 outputs. Numeric
facts are generally retained, but prompt compliance alone does not prevent
unsupported semantic additions. In particular, the editor's request to vary
sentence openings and smooth transitions introduced several violations.

| System | Finding | Required repair |
|---|---|---|
| 51 Peg | The title calls the planet a "Pioneering Hot Jupiter"; the body infers a "very tight orbit" and historical notability. | Use a neutral title and end with supplied mass and period facts only. |
| 55 Cnc | "Next" and "Farther out" infer orbital order from the request list. | Enumerate planets neutrally; retain only approved period rankings. |
| GJ 1214 | "Whips around" implies speed from an orbital period. | Say that the planet completes an orbit in its supplied period. |
| HD 10180 | "Whips around", "follows", "from there", and "Outermost" infer speed or physical arrangement. | State periods and approved rankings without spatial sequencing. |
| HD 189733 | The article preserves the age upper limit, radius-based class, and density correctly. | Keep as a concise one-planet reference case. |
| HD 41004 A | "Massive" is vague; the article is short for its target range. | Prefer the supplied minimum-mass wording; add only supported details if length matters. |
| HR 8799 | The article says HR 8799 e has a year longer than Earth's, although no period or comparison is supplied. "Young" is interpretive. | Omit the period claim for e; state the supplied age rather than an inferred age label. |
| K2-18 | Repeated "reported"/"estimated" and "roughly about" conflict with the editor brief. | Use one hedge, normally "about", while preserving Mass versus Msini. |
| Kepler-11 | The title calls the host Sun-like without a spectral type; its planet inventory is repetitive. | Use a neutral title; describe size classes as radius-based where needed. |
| Kepler-16 | All measurement values are unbolded. | Reject and retry until every used measurement phrase is bolded. |
| LHS 1140 | The final ranking paragraph repeats values already given; "confirmed" is not supplied. | Keep at most one useful ranking and say planets are associated with the host. |
| OGLE-2016-BLG-1195L | "Modest" is unsupported; the article derives uncertainty endpoints from raw errors and comments on a missing period. | Use only displayed mass, distance, host mass, discovery facts, and approved comparisons. |
| Proxima Cen | The final recap is redundant; "similar margin" is vague and unsupported. | End after the planet descriptions or a supported year comparison. |
| PSR B1257+12 | "Pulsar" is decoded from the discovery method; "Recognized second" is unsupported. | Use the object label or "stellar host" unless a host-type fact explicitly licenses pulsar language. |
| TRAPPIST-1 | "Moving outward" infers physical order; inventory prose is repetitive. | Enforce bolding, remove spatial language, and group selected approved distinctions. |

Draft comparison note: the saved `draft.md` files show the editor introduced
the position/speed wording ("moving outward", "whips", "farther out") while
the drafter introduced the 51 Peg notability claims; Kepler-16 is the
unbolded-measurements case (TRAPPIST-1's values were bolded).

## Generation Rules

### Evidence and semantic preservation

- Every reader-facing statement must be licensed by the prepared system,
  star, planet, approved-comparison, or included-guide facts.
- A period permits its stated duration and an approved year comparison. It
  does not permit orbital position, speed, distance, or ordering claims.
- Missing measurements are omitted. The article must not state that a value
  is unavailable, unreported, unlisted, or unmeasured. The one licensed
  exception is the prepared comparison fact "No orbital period is reported
  for {planet}.", which may be restated plainly.
- Raw `value`, `error_plus`, and `error_minus` fields are audit data. Their
  values must not be rendered unless they also appear in reader-facing
  display text or an approved comparison.
- Class, history, age-label, significance, and host-type language must be
  explicitly licensed. Curated licensed facts: a per-planet `circumbinary`
  flag (from NASA `cb_flag`), a `host_kind = "pulsar"` star fact plus the
  `Pulsar Timing` method label, an `age_class = "very young"` star fact for
  age estimates under 0.1 billion years, and "Sun-like" wording only when a
  G-type spectral label is supplied. Without the fact, the wording is banned.

### Editorial stage

The editor may shorten prose, remove repeated qualifiers, improve grammar,
and smooth genuinely neutral transitions. It must preserve each sentence's
factual claim and must not add scientific meaning.

The editor prompt must explicitly prohibit introducing spatial, temporal,
causal, historical, classificatory, or evaluative language. It must not
replace neutral period language with motion or position language.

The following expressions are regression examples for prohibited additions:
`whips`, `races`, `farther out`, `inner`, `outer`, `outermost`, `innermost`,
`moving outward`, `closest`, `farthest`, `follows`, `notable`, `pioneering`,
`young`, `modest`, and `recognized second`.

### Markdown rendering

- The response has one leading H1 and paragraphs only.
- Every occurrence of a planet name is bold.
- Every used measurement is bolded as one complete display phrase, including
  "about" where present, its number, and its unit. Spectral-type labels are
  also bolded when used.
- Unbolded text that matches an allowed measurement value and unit is invalid.

## Deterministic Gates

Run these gates after both the drafter and editor stages. A failure supplies
specific retry feedback and does not replace an existing successful article.

1. Markdown structure and prohibited punctuation gate.
2. Existing planet-name bolding gate.
3. Measurement-emphasis gate: identify rendered measurement phrases from the
   request and require any used phrase to occur inside a Markdown `strong`
   span.
4. Source-to-draft numeric allowlist gate. Output numeric tokens must be a
   subset of tokens licensed by display strings, discovery years, usable
   system counts, approved comparisons, included guide constants, and object
   names. Do not license raw audit-value/error tokens merely because they are
   serialized in the request.
5. Existing draft-to-editor numeric equality and planet-name-retention gate.
6. Policy-pattern gate for missing-data commentary, raw-uncertainty language,
   orbital position/speed wording, significance/hype, unlicensed labels
   (`pulsar`, `hot jupiter`, `circumbinary`, `sun-like`, `young`), the
   doubled hedge `about about`, meta/preparation language ("the guide",
   "supplied", "the request", "the evidence"), and per-request unlicensed
   labels. Pattern matches return the matched phrase and rule.
   Licensing-aware: phrases licensed by prepared facts (for example
   "pulsar timing" from a supplied method, "very young" from `age_class`,
   "sun-like" from a G-type spectral label) are removed from the text
   before the pattern list runs.

The numeric equality gate alone is insufficient: it detects an editor
altering a draft but cannot catch numbers first invented by the drafter, such
as OGLE's unlicensed uncertainty endpoints.

## Semantic Critic

Between the drafter and the editor, use a compact verifier call only for
residual semantic risks. It receives the draft plus normalized,
reader-facing allowed facts and returns strict JSON (json mode,
`response_format: json_object`, 768-token cap) with `findings` entries of
`category`, `quote`, `problem`, and `fix`, using one of these categories:

- `unsupported_fact`
- `position_or_speed_inference`
- `missing_field_commentary`
- `raw_uncertainty_derivation`
- `unlicensed_classification`
- `editorial_hype`

The contract is violations-only: an empty `findings` array when nothing
violates, and never a list of checks considered and cleared. One parse
retry; unparseable or transport-failed responses fail open. The verifier
does not rewrite prose; its findings are appended to the editor's input as
defects to resolve while polishing. The editor's output passes the same
deterministic gates plus fact preservation against the draft, so a finding
that would require dropping a number cannot be obeyed. The critic
contract: quotes verbatim from the draft, no placeholder findings,
licensed collectives and rankings are correct, and findings never mention
the guide/request/evidence/preparation. Known limitation (2026-09-10):
the flash model still produces false positives on licensed restatements;
the editor resists them and the gates bound the damage, but the
per-call hit rate is low.

## Algorithmic Formatting (2026-09-10)

Bolding is a presentation concern, never an LLM responsibility. Every
stage output passes `validate::normalize_article` before the gates: plain
`number+unit` phrases (including the full `times Earth's` /
`times Jupiter's` forms) and the supplied spectral label are wrapped in
bold when they appear outside strong spans in body text, and
`times Earth` is repaired to `times Earth's`. The pass is idempotent,
skips the title, and the bolding gate remains only as a backstop for
detector misses. `dev descriptions normalize` re-applies the same pass to
stored descriptions for retroactive fixes.

## Manual Notes

An optional hand-edited `notes.toml` beside the request carries
per-system intent: `facts` (merged into `publishable_comparisons`, so
their numbers license prose and the critic sees them as source) and
`guidance` (merged into `silent_constraints`). Merged in memory at
preflight; the fingerprint covers the merge, so edited notes regenerate
the system. Prepare and batch never write it. Nothing under
`content/systems/` is tracked: prose and notes are private local data
shipped to servers through the upload channel (see `specs/roadmap.md`).
Hand-editing `request.toml` remains ephemeral: `prepare --force`
clobbers it.

## Regression Cases

Tests must reject these known baseline failures:

- 51 Peg: "Hot Jupiter", "very tight orbit", and historical-significance
  prose without explicit support.
- 55 Cnc: "Farther out".
- HD 10180: "Outermost".
- HR 8799 e: an Earth-year comparison without a supplied period/comparison.
- OGLE-2016-BLG-1195L: the `1.11`--`1.88` raw-error range and a missing-period
  statement.
- PSR B1257+12: "pulsar" unless explicitly licensed in the request.
- Kepler-16: unbolded measurement phrases.
- TRAPPIST-1: "Moving outward".

Tests must also accept neutral replacements that preserve the exact displayed
numbers, units, provenance, bounds, planet names, and approved comparisons.

## Implementation Notes (2026-09-10)

The pipeline is draft-critic-edit (`batch.rs::generate_system`): the drafter
passes the deterministic gates, the saved `draft.md` goes to one critic call
(json mode, 768-token cap, one parse retry, fail-open), and the editor
receives the draft plus any findings plus the full evidence, passing the
gates plus `preserves_facts` against the draft. Editor failure fails the
system as `"editor"` while preserving the previous description. Gates 1-6
are implemented in `crates/exo-cli/src/descriptions/validate.rs`; the
licensed numeric set is collected in `batch.rs::collect_licensed` and
licensed phrases in `batch.rs::licensed_phrases`. Curated licensed facts
(`circumbinary` from `cb_flag`, `host_kind`, `age_class`, the no-period
fact) are emitted by `prepare.rs` with matching guide entries. A
`radial_velocity` guide entry licenses radial-velocity explanations.
