# Prose Generation Harness

## Purpose

This specification records the editorial quality requirements for generated
stellar-host descriptions. It supplements the CLI contract: source requests
remain the sole authority for reader-facing claims, and prose must not turn
measurements or request ordering into new scientific conclusions.

## Baseline Editorial Audit

The fixed 15-system baseline was reviewed against its saved `request.toml`
files and the shared drafter/editor prompts. The audit table below refers
to the earlier two-stage, fingerprint-version-2 outputs dated 2026-09-09;
pass 1 used draft-critic-edit fingerprint-version-4 outputs, and the
current pipeline is one writing pass plus optional repair, fingerprint-version-10. Numeric
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
- State orbital periods in days. Omit Earth-year comparisons and conversions
  entirely (user decision, v9); preparation stops emitting them and wire input
  filters legacy comparisons after notes merging. Approved shortest/longest
  period rankings among the listed planets remain available. A period does
  not permit orbital position, speed, distance, or ordering claims.
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
  The `very young` age class licenses both standalone `very young` and `young`,
  including in titles. It does not license `younger` or `youngest`; removing
  licensed age labels uses word boundaries rather than substring replacement.
- Express supplied host, planet, and star counts in natural reader-facing
  language. Do not expose catalog bookkeeping or matching-host terminology; a
  one-star count means a single star, not a stellar companion.
- The approved-comparison list is exhaustive. Do not add significance claims
  or mass rankings unless explicitly supplied, and never rank ordinary mass
  against minimum mass. These rules also apply to titles.
- Group shared comparisons instead of repeating the same sentence frame for
  each planet. Remove obvious grammar errors and tautologies without adding a
  mandatory style-edit stage; repetition analysis remains advisory.

### Writer and optional repair (v10)

Build compact reader-facing JSON after notes merging; omit audit values/errors
and source identifiers while preserving displayed values, bounds, measurement
kinds, approved comparisons, guide entries, and private guidance. Keep ordinary
mass separate from minimum mass without raw audit notation. Model-facing
measurements, comparisons, and guide entries omit measurement `about` prefixes.
Use plain values without adding an approximation note or disclaimer; preserve
minimum-mass wording and explicit upper/lower limits.

Normalize and validate the first response, installing it directly when valid.
Only a concrete validation failure triggers one repair with the failed article,
specific violations, and compact facts. Preserve unaffected supported content
while allowing invalid values/claims to be corrected. There is no always-run
editor or numeric-set equality requirement against an invalid draft. Transport,
malformed/empty, and truncated responses fail without automatic retry.

Target 20 valid articles within 100,000 reported tokens including failures and
repairs; the user accepts 60–100k, and lower usage is welcome. This is an
acceptance target, not a runtime budget setting. Keep current
model settings, soft length targets, and the fixed 20-system set. Evidence review
remains necessary.

### Historical style stage (v6/v7, retired)

The repetition editor receives the normalized draft plus the canonical,
notes-merged prepared request. The request is factual authority; audit fields
and silent constraints are not publishable. The drafter selects useful
measurements and names every planet, with a brief host introduction and compact
planet descriptions. Grouping is optional and the length target is soft.

The editor removes duplicate mentions, redundant qualifiers, repeated
explanations, inventories, and recaps while preserving every distinct supported
fact. It may combine sentences but must preserve numbers, units, planet/value
associations, bounds, and minimum-mass meaning. "About" expresses approximation;
an extra "estimated" is redundant. Explicit bounds are not redundant. Correct
or remove unsupported qualitative wording against the request, without adding
facts that the draft did not use. Consistent structure is acceptable.
Use compact `minimum mass` wording without raw `M sin i` notation; retain
distinct mass-provenance explanation once, removing its repetitions.

Numeric-set equality permits duplicate removal and detects added/lost values;
planet-name retention checks names mentioned in the draft. These checks do not
verify units, bounds, associations, complete planet coverage, or general semantic
equivalence. Targeted claim checks supplement them; evidence review is required
before benchmark acceptance.

The style prompt must explicitly prohibit introducing spatial, temporal,
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
  its number and unit. Measurement `about` prefixes are removed mechanically.
  Spectral-type labels are
  also bolded when used.
- Unbolded text that matches an allowed measurement value and unit is invalid.

## Deterministic Gates

Stage application (v10): writer and optional repair pass structural, numeric,
emphasis, coverage, and policy/claim checks. A system makes at most two calls.
The repair receives specific violations and the failed text. Repetition analysis
is advisory; exhausted validation preserves the existing successful article.

1. Markdown structure and prohibited punctuation gate (em dashes and
   double hyphens are also repaired mechanically first — see
   Algorithmic Formatting — so this fires only as a backstop).
2. Planet-name bolding gate (also applied mechanically first; the
   gate is a backstop).
3. Measurement-emphasis gate: identify rendered measurement phrases from the
   request and require any used phrase to occur inside a Markdown `strong`
   span (also applied mechanically first).
4. Source-to-draft numeric allowlist gate. Output numeric tokens must be a
   subset of tokens licensed by display strings, discovery years, usable
   system counts, approved comparisons, included guide constants, and object
   names. Do not license raw audit-value/error tokens merely because they are
   serialized in the request.
5. Every requested planet name must appear. Numeric equality against a draft
   is retired: repair may correct an invalid value using supplied facts.
6. Policy-pattern gate for missing-data commentary, raw-uncertainty language,
   orbital position/speed wording, significance/hype, unlicensed labels
   (`pulsar`, `hot jupiter`, `circumbinary`, `sun-like`, `young`), the
   doubled hedge `about about` (also repaired mechanically first),
   meta/preparation language ("the guide",
   "supplied", "the request", "the evidence"), and per-request unlicensed
   labels. "Consistent with" (unsupported interpretation) and "not constrained"
   (missing-data commentary) remain policy patterns. Pattern matches return
   the matched phrase and rule.
   Licensing-aware: phrases licensed by prepared facts (for example
   "pulsar timing" from a supplied method, `very young`/`young` from `age_class`,
   "sun-like" from a G-type spectral label) are removed from the text
   before the pattern list runs.
7. Omit Earth-year comparisons and conversions, even when present in legacy
   requests. Retain the stated orbital periods in days. The former per-planet
   subject/direction licensing logic is retired. Reject the observed stellar minimum
   mass error. In multi-star requests, reject
   explicit system-wide attribution of the selected star's age, mass, radius,
   or temperature. These are bounded wording checks, not a general semantic
   parser. Failures identify the claim and request a named supported replacement.

The numeric equality gate alone is insufficient: it detects an editor
altering a draft but cannot catch numbers first invented by the drafter,
such as OGLE's unlicensed uncertainty endpoints.

### Historical Narrative Gates (v5, retired 2026-09-14)

The v5 pipeline collected narrative violations and forwarded them to the
style pass. These gates are retired: density, opener count, sentence-length
spread, and number reuse do not reject or retry articles. Existing corpus
analysis remains advisory. "Associated with" and "sin i" are prompt wording
guidance, not rejection patterns. The former thresholds below document v5 only.

7. Narrative gates: (a) one word may not open more than 40% of
   sentences (or 8 sentences absolutely) in articles of five or more
   sentences, counting the first word that is neither a number nor a
   planet-name/hostname token; (b) sentence word counts must vary by a
   population standard deviation of at least 2.0 in such articles;
   (c) numeric density must stay at or below 2.5 numbers per sentence;
   (d) banned machine phrasings: "associated with" (any tail — the
   inventory formula), "sin i" (raw audit notation), "consistent with"
   and "not constrained" (interpretive/missing-data drift observed in
   R5);    (e) a number+unit pair may appear at most twice outside object
   names (hostname and planet names) and the spectral label, except
   the 365-day Earth-year constant — two planets may legitimately
   share a value, but a third occurrence marks a recap; distinct
   quantities that merely share a number ("8 billion years" vs
   "8 Earth masses") never count as reuse.

## Semantic Critic (removed 2026-09-12)

The draft-critic-edit pipeline used a JSON-mode verifier between the
drafter and the editor. Pass-1 measurement showed it cost 13.5% of
prompt tokens while its findings were mostly false positives the editor
resisted and the gates bounded. It was removed for the two-pass
pipeline (fingerprint v5). V6 gives the repetition editor the prepared request
and retains numeric/policy checks plus manual evidence review; none is a
complete semantic verifier. The
historical contract is preserved below for interpreting pass-1
metadata, which records a `critic` table on systems where it ran.

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

## Algorithmic Formatting (2026-09-10; extended 2026-09-12)

Presentation is never an LLM responsibility. Every stage output passes
`validate::normalize_article` before the gates:

- Em dashes and double hyphens become single hyphens: digit-adjacent
  dashes collapse to a plain hyphen (numeric ranges), all others become
  spaced hyphens.
- Any run of repeated `about` first collapses to one occurrence.
- Redundant `estimated` hedges collapse before measurement contexts:
  before radius/radii/mass(es)/mean density/density/age/temperature/
  period/distance/year(s)/minimum-mass quantity, before `about` and
  bold spans ("an estimated **about X**" → "**about X**"), and in the
  verb form ("is estimated at X" → "is X"); article agreement is kept
  ("an estimated radius" → "a radius"). Other uses are untouched.
- Plain `number+unit` phrases (including the full `times Earth's` /
  `times Jupiter's` forms), the supplied spectral label, and every
  planet-name occurrence (longest first) are wrapped in bold outside
  existing strong spans; the title line is skipped.
- Measurement matching includes adjacent `about` and complete units, including
  the superscript in `g/cm³`. Repair strong boundaries over complete visible
  measurement phrases, including already-split units. The emphasis gate checks
  complete phrases across Markdown text events rather than isolated text runs.
- Remove measurement `about` prefixes after formatting, including ranges,
  lists, spelled numbers, and `about every <period>`. Keep ordinary prepositions
  such as `about its composition` and references such as `about 51 Peg`.
  Do not change values, units, upper/lower limits, or minimum-mass labels.
- `times Earth` is repaired to `times Earth's`, consuming an ASCII or
  typographic possessive suffix if already present.

The pass is idempotent. `dev descriptions normalize` re-applies it to served
descriptions; `normalize --label pass2` selects only `description_pass2.md`.
Only changed articles are written. Preparation, generation metadata, drafts,
and other variants are untouched; no model calls or re-preparation are needed.
The corresponding gates remain as backstops for detector misses.

## Manual Notes

For an explicitly selected local variant, run
`just copy-descriptions description_pass2` from any shell, including fish.
The required argument is a `description_<label>` filename stem without `.md`.
The recipe checks all matching articles and metadata are nonempty and have no
matching latest-failure markers before copying. It replaces `description.md`
and `metadata.toml` in each matching system directory, preserves the variant
files, and prints the number copied. No matches or invalid input is an error.
Restart the website afterward; descriptions are loaded only at startup.

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

## Implementation Notes (2026-09-15)

`batch.rs::generate_system` runs one writer call and optionally one repair,
fingerprint v10. `batch/input.rs` projects compact facts after notes merging
and filters retired Earth-year comparisons from legacy requests/notes.
`draft.md` retains the normalized first response; only a validated candidate is
installed with metadata. The repair prompt is
`content/stellarhost_repair_prompt.txt`, overridden by `--repair-prompt`
with legacy `--style-prompt` alias. New stages are `draft` and optional
`repair`; historical `style` records remain readable.

`usage.rs` aggregates attempted stages independently of validation success.
Retain usage from rejected responses and explicitly mark incomplete usage.
Analysis/status prefer latest matching failures and recover historical
undercounts from stage records without rewriting stored files. Gates and
normalization remain in `validate.rs`; curated preparation facts and guide
selection remain in `prepare.rs`.

De-monotonization round 1 (2026-09-12): `prepare.rs` gates the
Earth-year comparison to year extremes (see Generation Rules); the
drafter prompt requires varied planet-inventory phrasing (no fixed "N
planets are associated with this host" formula) and at most one
discovery-method explanation per article. `dev descriptions analyze
compare --baseline-dir` diffs two corpora on headline metrics and
trope reach for evaluating regenerations.
