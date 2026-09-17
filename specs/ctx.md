# Current Task Context: Generated System Descriptions (#116)

State: in progress — `v10-tuned` completed and is materially improved. Awaiting
user acceptance before the off-peak full-catalog generation.

Disk state verified 2026-09-17: served articles still contain 39,054 `about`
occurrences; `description_pass2.md` contains only 23 non-measurement uses.
All 4,768 served/variant pairs currently differ; `pass2` has zero failure files.
The user already promoted the original `pass2` and viewed the site before this
cleanup. Old backups remain at `tmp/pass1-snapshot` and `content/stats/pass1.json`.

## Plan

### Current: plain measurement values
- [x] Add `just copy-descriptions description_pass2` for the user's fish shell;
      copy articles and matching metadata with visible progress/counts.
- [x] Remove measurement `about` mechanically in generation input and output;
      keep numbers, units, bounds, minimum masses, and ordinary prepositions.
      No approximation note/disclaimer. Fingerprint v10; `normalize --label` added.
- [x] Verify 96 prose tests; normalize all `description_pass2.md` files and
      compare them against still-unchanged served files.
- [x] Run a fresh labeled v10 experiment on the fixed 20-system set during
      DeepSeek off-peak hours; review failures, usage, and every final article.
- [x] Add focused writer/repair editorial rules, fix Clippy, then regenerate
      and review the fixed 20-system set under the fresh `v10-tuned` label.
- [ ] If accepted, regenerate the full catalog during off-peak hours, retry
      failures, compare and promote the result, then inspect the website.

### Draft → repetition edit (2026-09-14)
- [x] R7: fixed 20-system experiment facility with refreshed requests,
      labeled artifacts, and hostname-intersection comparisons.
- [x] Update prompts and editor input: draft selects coverage; editor
      receives draft plus merged request, removes redundancy, and retains
      every distinct supported fact. Keep `style` interfaces; fingerprint v6.
- [x] Remove rhythm/density/reuse rejection and defect forwarding; retain
      formatting, numeric-retention, name, and semantic-pattern checks.
- [x] Fix labeled failure reporting/comparison exclusion and expose
      `generate-batch --label ... --failed` for bounded retries.
- [x] Verify focused prose/CLI regressions and served-output preservation.
- [x] R5 experiment: user ran `simplify-v1`; reviewed all 20 draft/edit pairs
      against their requests and compared with served output and v2.
- [x] R5a: make request-derived validation reject an Earth-year comparison
      unless the exact planet has the matching approved comparison; reject
      star measurements described as system-wide in multi-star requests.
- [x] R5b: tighten the editor prompt and normalizer for the observed output:
      no unused request facts, no raw `M sin i`, no redundant inventory/recap,
      compact minimum-mass wording, and intact bold measurement units.
- [x] R5c: replace the empty `--failed` selection error with a clear
      "No failed systems" outcome; preserve its no-network behavior.
- [x] R5d: add focused regressions for the listed cases, then run a new labeled
      20-system experiment and review only changed/failing articles.
- [x] R8a: compact reader-facing JSON, one writer call, and at most one repair
      receiving the failed article, violations, and facts; fingerprint v8.
- [x] R8b: named-clause year validation, stellar minimum-mass and coverage
      checks, complete attempted-stage accounting, historical failure totals,
      `repair` metadata, and `--repair-prompt` with legacy CLI alias.
- [x] R8c: focused tests and local smoke; fixed `onepass-v1` experiment;
      compare against `simplify-v1` and review all 18 articles plus two failures.
- [x] R9: omit Earth-year comparisons from preparation, legacy wire inputs,
      and writer/repair prompts; replace subject licensing with an omission
      check, retain periods in days and approved period rankings; fingerprint v9.
- [x] R10: permit standalone `young` when `age_class = "very young"`,
      including titles; keep `younger`/`youngest` rejected. Verify focused
      regressions and retry only the final `pass2` failure.
- [ ] R5 acceptance: resolve the observed review defects below. Acceptance needs
      20 current successes, no observed factual defects, and no recurring
      redundant-qualifier or duplicated-explanation pattern. The user accepts
      60–100k reported tokens per 20-system experiment including repairs/failures;
      lower usage is welcome.
- [ ] R6: full pass 2 — `prepare --all --force`, `generate-batch`
      only after benchmark acceptance; `--failed` passes,
      `compare --baseline-dir tmp/pass1-snapshot`, snapshot
      `content/stats/pass2.json`, record totals

## Findings

- `v10-tuned` (2026-09-17): 20 successes, zero failures, 23 calls, 45,247
  reported tokens with complete usage, and 15.10 seconds wall time. Seventeen
  articles passed directly; three narrowly repaired `innermost`/`outermost`
  wording without dropping surrounding facts. Against `v10-test`, mean words
  fell 226.9→222.85, median 214→202, and TTR rose 0.47→0.48. The two duplicate
  sentences are harmless five/six-planet count lines; measurement bolding and
  Earth-year omission remain clean.
- The tuned prompt fixed the reviewed substantive defects in 55 Cnc, HR 8799,
  51 Peg, GJ 1214, Kepler-16, TOI-178, and TRAPPIST-1. Residual copy edits are
  HD 189733's still-unlicensed "Close-Orbiting" title and "an mean density" in
  HR 8799 and LHS 1140. Kepler-42's "Their years are short" and TOI-178's
  "fall between these in period" are minor unsupplied qualitative/derived
  wording. No further model iteration is recommended; these are cheaper to
  catch and edit after the full batch.
- Focused follow-up (2026-09-17): tightened only writer/repair guidance for
  natural count language, licensed significance/rankings, mixed mass provenance,
  repeated frames, grammar, and tautology. Removed the abandoned defect-marker
  plan. Fixed both reported Clippy findings; `cargo fmt --all`, all 96 prose
  library tests, `cargo lx`, and `git diff --check` pass. Selective regeneration
  of the nine reviewed `v10-test` articles is pending the next off-peak window.
- User decision (2026-09-17): v10 is good enough overall; do not add defect
  tracking or begin another broad prompt/gate iteration. Add small editorial
  rules and selectively regenerate the reviewed articles by hostname.
- `v10-test` (2026-09-17): 20 successes, zero failures, 27 calls, 47,870
  reported tokens with complete usage, and 17.45 seconds wall time. Thirteen
  articles passed directly; seven were repaired. Automated analysis found no
  unbolded measurements or Earth-year comparisons and one cross-system duplicate.
  Repairs were narrow and preserved unaffected facts.
- Manual review rejects benchmark acceptance. 55 Cnc adds the unsupported
  "most thoroughly studied" claim and an explicitly prohibited mixed-provenance
  "most massive" ranking. HR 8799 turns a one-star catalog count into a stellar
  companion. HD 189733 infers "close-orbiting" in the title. 51 Peg and GJ 1214
  leak internal catalog-count language. TRAPPIST-1 repeats the same comparison
  sentence pattern seven times; Kepler-11 adds a redundant comparison paragraph;
  Kepler-16 is tautological; TOI-178 contains "an mean density". Do not start the
  full batch or another paid experiment until these defects are addressed.
- User decision (2026-09-17): do not promote the mechanically cleaned old
  `pass2` corpus yet. First validate the complete current v10 generation path
  with a fresh 20-system experiment; if accepted, regenerate the full catalog
  during DeepSeek off-peak hours.
- Session handoff (2026-09-17): the user approved the displayed prose except
  repeated `about`. They rejected an approximation note/disclaimer and asked
  for plain measurement values. Keep explicit minimum mass and upper/lower
  limits. Current scope is mechanical cleanup; do not restart model experiments
  or unrelated prose-quality work based on older findings below.
- The user uses fish and wants plain commands, not shell functions. The
  Justfile now has `just copy-descriptions description_pass2`, implemented
  with a Bash shebang so the caller's shell does not matter. It validates the
  selected articles/metadata and failure markers before copying, preserves
  variant files, and prints counts. The user ran the promotion and viewed it.
- Current implementation: `validate.rs::remove_measurement_about` handles
  numeric values, spelled quantities, ranges/lists, comma-separated numbers,
  written density units, and `about every <period>`, including Markdown spans.
  It preserves ordinary phrases such as `about its composition`, object-name
  references, and qualitative comparisons such as `about the same age`.
  The same cleanup is used in compact wire facts and after article formatting.
  Writer/repair prompts request plain values; no model calls were made.
- `dev descriptions normalize` processes served articles; `normalize --label
  pass2` processes only the selected variant. Requests, metadata, drafts, and
  other variants are not rewritten. Generation metadata stays at its original
  version; v10 is for future generation. Do not call `generate-batch` to apply
  this cleanup: changed prompts/fingerprints could cause paid regeneration.
- Completed cleanup: 4,768 variants rewritten on the first pass, then 30 on
  the refined edge-case pass. `about` count is 39,054→23 (39,031 measurement
  prefixes removed). Remaining cases were inspected: ordinary prepositions
  plus one qualitative comparison. Existing normalization also removed redundant
  `an estimated`/`estimated at` wording in 11 articles. Per-file comparison
  found no changes beyond those measurement hedges; numbers, units, bounds,
  minimum-mass labels, and all other wording were preserved.
- Verification: `cargo fmt --all`, all 96 `exodata-prose` library tests, native
  labeled normalization, and `git diff --check` passed. Tests cover ordinary
  `about`, quantity ranges/lists, written units, Markdown, bounds, idempotence,
  and label isolation. The final served-normalization command was interrupted;
  the disk check above confirms served files still need cleanup. No full
  regeneration, metadata rewrite, staging, commit, or deployment is required.

- Pipeline economics (rebuilt, 2026-09-12): ~2 calls/system
  (draft + style), 3 attempts/stage max; baseline-15 runs land at
  3-7k tokens/system vs 8.4k pass-1 mean (3.02 calls/system:
  draft 1.47, critic 0.47, editor 1.08; critic removed at 13.5%
  of prompt tokens for mostly false-positive findings).
- Pass-1 corpus stats (see `content/stats/pass1.json`): median 148
  words / 8 sentences, TTR 0.58; monotony — 365-day anchor 92%,
  "one planet is associated with this host" verbatim in 1,173,
  listed-planets ranking 929; 1,370 cross-system duplicate
  sentences. Algorithmic bolding clean (10 one-off residuals).
- Earlier baseline review found improvements in explainers/grouping, but
  output year-anchor licensing is not exact: saved v2 gives TRAPPIST-1 g/h
  unapproved Earth-year comparisons. Numeric sets cannot verify associations,
  units, bounds, minimum-mass wording, or semantic comparisons.
- Experiment-v2 run 1 (2026-09-13, 16/20): comparing with labeled
  corpora requires hostname intersection (fixed in `analyze compare`);
  failures were two distinct-quantity pairs sharing value+unit
  (reuse cap then raised to 2) and density/opener failures on 6-7-planet
  systems; the qualifier prompt line backfired (mentioning
  "estimated" taught it: HD 10180 1→16) — replaced with positive
  phrasing plus a spread-measurements rule.
- Known failure semantics: failures preserve the previous
  description; `--failed` overrides the fingerprint skip; staging
  leftovers block preflight.
- Pre-change v2 inspection (2026-09-14): 20 saved articles but newer failures
  for HD 41004 A (unlicensed "young") and TRAPPIST-1 (density); analysis
  incorrectly reported zero failures because it read unlabeled failure files.
  Saved-corpus comparison: duplicates 3→0, unbolded measurements 0,
  median words 262→234. These numbers include the older successes.
- Agreed simplification: consistent structure is acceptable; the editor cuts
  redundancy, not distinct supported facts. It receives the prepared request
  for checking meaning. "About" expresses approximation; bounds and minimum
  masses remain explicit. No third stage or replacement scoring framework.
- Verification (2026-09-15): all 86 `exodata-prose` tests and the focused
  `exodata` labeled-retry CLI test passed; `cargo fmt --all` and
  `git diff --check` completed. Live local v2 comparison now reports 2 failed
  systems and 18 successful pairs (duplicates 2→0, unbolded measurements 0).
  No served descriptions were written.
- User ran `simplify-v1` manually, resolving the earlier credential blocker:
  20 successes, zero failure files; 19 systems used two calls and HD 260655
  used three (editor introduced 365 on its first attempt). Saved usage totals
  135,022 tokens, mean 6,751/system. No batch retry was needed; `--failed`
  returned the misleading "No prepared requests found" for an empty selection.
- Benchmark comparison: served→simplify-v1 mean words 283.9→229.85,
  median 262→218, duplicates 3→2, unbolded-measurement detector 0→0.
  Against v2's 18 nonfailed pairs: mean words 263→225.94, duplicates 0→2.
  The duplicates are ordinary one/five-planet count sentences, not a blocker.
- Review of all 20 draft/edit pairs (2026-09-15): every current request planet
  is present; no changed numeric measurements or lost bounds observed.
  No final article contains "estimated". K2-18/GJ 1214 show useful cleanup;
  HD 10180 still repeats "reported minimum-mass quantity" six times, Proxima
  retains raw "M sin i", and LHS 1140 repeats the mass-vs-minimum explanation.
- Acceptance defects retained from drafts: HD 41004 A assigns its host-star
  age upper limit to the system; L 98-59 e/f and TOI-178 f/g receive unlicensed
  Earth-year comparisons (TOI via an "Every planet" closing statement).
  Editor scope misses: it adds a licensed radius comparison in HD 189733,
  a licensed radius ranking in TOI-178, and a redundant inventory in V1298 Tau.
  V1298 Tau retains split bold unit text (`g/cm**³`), missed by the detector.
  Keep the simpler architecture; benchmark sign-off remains open.
- R5a–R5c implemented (2026-09-15): request-derived checks use the exact
  planet's approved Earth-year direction and reject unnamed/blanket subjects;
  explicit host-star measurements assigned to a multi-star system are rejected.
  Prompts reinforce coverage preservation and compact minimum-mass wording.
  Normalization includes adjacent `about` and complete units across split bold
  boundaries; the emphasis gate checks complete visible measurement spans.
  Empty unfiltered labeled/unlabeled retries return `No failed systems` and
  empty results without credentials, calls, or writes. Fingerprint is v7;
  request schema and `style` interfaces are unchanged.
- Verification: 89 prose library tests and two focused CLI tests passed;
  the final span-check adjustment passed all 20 validation tests. The local
  empty-retry smoke returned `No failed systems` and `[]`. `cargo fmt --all`
  and `git diff --check` completed. Served articles and every `simplify-v1`
  artifact matched the pre-experiment SHA-256 aggregate after the run.
- `simplify-v2` attempt: 0 generated, 20 failed, 302.83 seconds. All failures
  recorded HTTP 200 followed by response-read failure at approximately 60
  seconds, before any draft text or usage report arrived. A single 51 Peg
  retry at concurrency 1 failed the same way. No usage was returned; reported
  zero tokens is not evidence of zero provider billing. No new draft/edit
  pairs exist to review or compare. Keep R5d and acceptance open; do not
  change the timeout/model or keep retrying the whole batch to hide this blocker.
- User-authorized retry (2026-09-15): DeepSeek responded normally; 16 generated,
  four validation failures, 38.62 seconds, 66 calls, 217,181 reported tokens
  (10,859/system including failures). K2-18 and L 98-59 exhausted style retries
  because the editor inserted `365` absent from their drafts. TOI-178 exhausted
  style retries on an unapproved Earth-year comparison for f; TRAPPIST-1
  exhausted draft retries on unnamed comparisons, including unapproved g/h.
  These are generation/validation defects, not further transport failures.
- Reviewed all 16 successful draft/edit pairs and the four failed systems'
  saved drafts/returned text. All 16 final articles name every request planet;
  no raw `M sin i`, `estimated`, or split `g/cm**³` remains. Proxima minimum-mass
  wording and V1298 Tau unit formatting improve. LHS 1140 no longer repeats
  the mass-versus-minimum explanation, but HR 8799 repeats it four times.
- New factual defect: HD 41004 A's editor turns the star's ordinary mass
  estimate (`about 0.95 solar masses`) into a minimum mass. Its age upper limit
  now belongs to the star correctly, but the editor also drops the explicit
  one-planet count. Numeric/name gates do not catch this meaning change.
- Editor still adds unused approved facts: second-shortest/second-longest
  year rankings for 55 Cnc b/f, an Earth-year comparison for HD 10180 c, and a
  radius comparison for HD 189733 b. Kepler-42 retains repeated radius
  comparisons. Benchmark acceptance remains open despite formatting improvements.
- Gate limitation revealed by retry feedback: explicitly named clauses such
  as "its orbital period is about 4.23 days, and 51 Peg b's year is shorter
  than Earth's roughly 365-day year" are rejected because the sentence does
  not start with the planet name. The user chose explicit comparison subjects;
  accepting an unambiguous named clause would avoid these unnecessary retries
  without licensing pronoun-only or blanket comparisons.
- Comparison against `simplify-v1` uses 16 successful shared systems: mean
  words 201.81→211.63, median 187→211, duplicate sentences 1→1, unbolded
  measurement detector 0→0. The served articles and every `simplify-v1`
  artifact retain their pre-retry SHA-256 aggregate. No code or prompts were
  changed during this retry; no promotion or full-catalog generation occurred.
- Accounting correction found during v8 planning: `simplify-v2` headline
  totals omitted draft usage when style failed. The actual stage-record total
  is 249,027 tokens across 66 calls, not 217,181. Native analysis now reports
  the corrected total without rewriting the old artifacts.
- V8 implementation: compact JSON omits audit values/errors and source details,
  distinguishes `mass`/`minimum_mass`, and preserves displayed bounds and notes.
  A valid normalized draft is installed directly; one repair is allowed after
  validation failure, with actual failed text and compact facts. No always-run
  editor or equality check against an invalid draft remains. New metadata uses
  `draft`/optional `repair` and `usage_complete`; old `style` metadata remains
  readable. `--repair-prompt` has a legacy `--style-prompt` alias.
- Verification: the 91-test prose suite passed, followed by the added repair
  transport-accounting regression (92 prose tests total), and both focused CLI
  tests passed. The no-work smoke returned `No failed systems` and `[]`.
  Historical `simplify-v2` analysis independently reported 249,027 tokens.
  Formatting/diff checks completed. Hashes confirm served articles and all
  `simplify-v1`/`simplify-v2` artifacts are unchanged.
- `onepass-v1`: 18 automated successes, two failures, 33 calls, 25.28 seconds,
  66,608 tokens with complete reported usage. Seven systems passed first call;
  thirteen required repair (eleven repaired successfully). Writer usage was
  36,244 tokens; repair usage was 30,364. This is 73% below corrected v7 usage,
  but 6,608 above the accepted target. HR 8799 failed on an unnamed Earth-year
  comparison; TRAPPIST-1 failed on unapproved comparisons for g/h. No further
  paid rerun was made after this missed benchmark.
- Review of all 18 final articles and the two failed responses: all final
  articles name every requested planet, but factual/editorial defects remain.
  HD 189733 retains unsupported "Sweltering" and "extremely close" wording;
  Proxima adds a remembered "closest known star to Earth" claim. 55 Cnc d
  becomes "the most massive" despite suppressed mixed-provenance rankings;
  HD 10180 also turns minimum-mass comparisons into true-mass wording and adds
  unsupported relative mass claims. HD 41004 A's stellar minimum-mass error
  is gone, but it adds the unapproved stellar mass comparison with the Sun.
- Repair preservation defect: HD 260655's repair removes the supported
  2.77-day and 5.71-day periods while fixing Earth-year subjects. Other output
  retains inventories/recaps (V1298 Tau, TOI-178, K2-18) and preparation wording
  such as "matched to this host". Comparison over 18 shared successes:
  mean words 214.89→234.67, median 187→227, duplicate sentences 2→0, unbolded
  detector 0→1 (HD 10180 restates stellar mass as "about 1.06 times that of
  the Sun"). Lower token cost is not benchmark acceptance.
- User decisions after v8 review: 60–100k tokens is acceptable; omit all
  Earth-year comparisons/conversions and keep periods in days. V9 removes
  comparison generation and its thresholds, filters legacy request/notes
  comparisons from compact input and numeric licensing, and replaces the
  per-planet subject/direction machinery with a simple omission check. General
  planet-year wording and approved period rankings remain supported. All 92
  prose tests passed; no new paid run or stored-article rewrite was made.
- V9 `onepass-v2` subsequently ran the full prepare/generate workflow for 20
  systems: 18 automated successes, two failures, 28 calls, 53,596 tokens.
  Both failures (HR 8799 and V1298 Tau) were the overly strict `young` label
  check. The catalog preparation audit found 4,769 source hosts, with 4,768
  preparable; 2MASS J11011926-7732383 has no usable stellar-host summary row.
- User ran the full labeled `pass2` generation and failed-only retries. The
  last remaining failure, 2MASS J16262785-2625152, explicitly supplied
  `age_class = "very young"`, but its title's standalone `Young` was rejected.
  The gate now accepts the weaker label with word boundaries when that class
  is present; missing age-class facts and `younger`/`youngest` stay rejected.
  Two focused regressions passed, covering title/body wording, request-derived
  licensing, and comparative/superlative rejection. Formatting/diff checks passed.
- Targeted retry (2026-09-16): only 2MASS J16262785-2625152 regenerated, passing
  on one call with 1,085 reported tokens. Its failure file cleared; `pass2`
  now contains 4,768 articles, 4,768 metadata records, and no failure files.
  Hashes confirm all served articles and every other `pass2` article/metadata
  record are unchanged. No global fingerprint bump or full regeneration was
  needed for this conditional licensing fix.

## Context

- Pipeline contract, gates, and mechanical post-pass:
  [prose_harness.md](prose_harness.md); CLI contracts:
  [cli.md](cli.md). Keep the `prose-generation` skill current.
- Fixed 15-system baseline (do not silently replace): 51 Peg,
  HD 41004 A, Kepler-11, LHS 1140, TRAPPIST-1, HR 8799,
  PSR B1257+12, Kepler-16, 55 Cnc, Proxima Cen, HD 189733,
  GJ 1214, K2-18, HD 10180, OGLE-2016-BLG-1195L. The experiment
  set adds Kepler-42, L 98-59, TOI-178, V1298 Tau, HD 260655.
- Content layout: nothing under `content/systems/` or
  `content/stats/` is tracked; prompts and the guide at `content/`
  root stay tracked. User handles all Git staging/commits.
- Keep the existing soft length targets, permitting shorter supported text.
  Mixed-provenance mass rankings remain suppressed (55 Cnc).
- Current boundaries: retain one writer plus optional repair. The user has
  authorized a fresh 20-system v10 experiment and, if accepted, an off-peak
  full regeneration. Preserve served prose, older variants, and backups until
  promotion. Do not add a critic, disclaimer, or unrelated editorial changes.

## Next

Confirm whether `v10-tuned` is accepted. If accepted, run the full preparation
and labeled generation during DeepSeek off-peak hours, then retry failures,
analyze anomalies/templates, review flagged prose, and promote only after that
post-pass. Preserve served prose and all older variants meanwhile.
