# Current Task Context: Generated System Descriptions (#116)

State: catalog pass 1 complete — all 4,768 preparable systems generated
(~39.9M recorded tokens across generation plus `--failed` retry passes;
zero standing failures; one permanent skip, 2MASS J11011926-7732383).
Round A (lines output, algorithmic formatting) done. Round B below is
the active plan. Completed work is archived in `CHANGELOG.md` 2026-09-10.

## Plan

### Round A — output and algorithmic formatting
- [x] `lines` output format (one compact line per system) as the default
      for `generate-batch`, `status`, `prepare`, and `normalize`;
      table/json/csv stay via `--output`
- [x] Algorithmic bolding: a post-pass wraps detected measurement phrases
      (number+unit and the spectral label, outside strong spans, body
      only) in bold before validation; the gate remains as backstop.
      Bolding is no longer an LLM responsibility
- [x] Possessive normalizer: `times Earth` → `times Earth's` in the same
      post-pass
- [x] `dev descriptions normalize` subcommand applying both normalizers
      to existing files; retro-fix run: 4,494 scanned, 258 rewritten,
      zero possessive slips remain
- [x] User `--failed` passes cleared all 274 standing failures; final
      state 4,768/4,768 generated, zero fail.toml files, ~39.9M recorded
      tokens total

### Round B — prose tooling rebuild
- [ ] P1: new `exo-prose` crate (normal workspace member) owning the
      descriptions pipeline (prepare, batch, validate, client, status,
      analyze, normalize) with its own LLM deps; `exo-cli` keeps command
      wiring; web backend untouched
- [ ] P2: rig-core provider abstraction; provider profiles (base URL,
      model, key env) selected with `--profile`; metadata records
      provider and served model; A/B-friendly
- [ ] P3: critic stays advisory but loses JSON mode — plain-text findings
      (one per line), no `response_format`; parse failures degrade to
      fail-open; bolding never LLM-checked
- [ ] P4: `--variants N` — N independent pipeline runs per system, all
      kept (ignored `variants/`) with per-variant metadata (tokens,
      attempts, profile); `description.md` = auto-selected cleanest
- [ ] P5: de-monotonization — prepare emits the Earth-year comparison
      only for year extremes; prompt limits the 365-day anchor; deeper
      fixes chosen from n-gram data
- [ ] P6: `dev descriptions analyze` — word/sentence counts, type-token
      ratio, top n-grams (1–5) with document frequency, trope report;
      the A/B evaluation instrument
- [ ] P7: regeneration campaign with the rebuilt pipeline; record totals

## Findings

- Final catalog state (2026-09-10): 4,768/4,768 generated, 0 failed,
  ~39.9M recorded tokens; the first pass produced 4,494 with 274 gate
  failures (183 bolding-gate, 68 editor fact-loss, 17 banned phrasing,
  5 numeric, 1 em-dash), cleared across a few `--failed` passes once
  algorithmic bolding landed. Mean ~8.4k tokens/system; ~47% of systems
  needed retries before the normalizer — the main Round B efficiency
  target.
- Critic (JSON mode): 15.5% of prompt tokens; findings on 2,075 systems,
  mostly false positives on licensed restatements; the editor resists
  them and gates bound the damage. Cache hit rate ≈ 75% of input tokens.
- Known prose warts, quantified on pass 1: "times Earth" missing
  possessive — fixed retroactively (258 files rewritten, zero remain);
  trivial "among the listed planets" rankings in 191/711 two-planet
  systems; per-planet inventory monotony (the standing length/compression
  open decision).
- Failure semantics verified live: failures preserve the previous
  description, `--failed` overrides the fingerprint skip (force-trap),
  staging leftovers block preflight until cleared.

## Context

- Pipeline contract and gates: [prose_harness.md](prose_harness.md);
  CLI contracts: [cli.md](cli.md). Keep the `prose-generation` skill in
  sync in `.agents` and the ignored `.claude` mirror.
- Fixed 15-system baseline (do not silently replace): 51 Peg, HD 41004 A,
  Kepler-11, LHS 1140, TRAPPIST-1, HR 8799, PSR B1257+12, Kepler-16,
  55 Cnc, Proxima Cen, HD 189733, GJ 1214, K2-18, HD 10180,
  OGLE-2016-BLG-1195L.
- Content layout: `description.md` + `notes.toml` tracked; request,
  metadata, evidence, draft, and failure files ignored. User handles all
  Git staging/commits.
- Baseline untrack command (user-run, still pending if not done):
  `git rm --cached content/systems/*/request.toml content/systems/*/metadata.toml`
- Open decisions: mixed-provenance mass ranking (55 Cnc) — current output
  avoids it; article length target; serving/deployment packaging;
  translation pipeline (few weeks out, after pass-2 content lands).

## Next

Start Round B with P1 (exo-prose crate split), then P2 (rig-core
providers).
