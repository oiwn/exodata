# Current Task Context: Generated System Descriptions (#116)

State: catalog pass 1 complete — all 4,768 preparable systems generated
(~39.9M recorded tokens across generation plus `--failed` retry passes;
zero standing failures; one permanent skip, 2MASS J11011926-7732383).
Round A (lines output, algorithmic formatting) done. Round B below is
the active plan, reprioritized 2026-09-11: crate split and analysis
tooling first, rig-core and content improving after, deployment last.
Completed work is archived in `CHANGELOG.md` 2026-09-10.

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

### Round B — prose tooling and content improvement (reprioritized 2026-09-11)
- [x] B1: `exodata-prose` crate split — package `exodata-prose`, lib
      `exo_prose` at `crates/exo-prose`; the descriptions pipeline
      (prepare, batch, validate, client, status, normalize, probe) moved
      out of exo-cli with its unit tests; exo-cli keeps command wiring
      and the assert_cmd CLI integration tests; `dataframe_to_json`
      moved to `exo_core::json`; no behavior change
- [ ] B2: analysis suite — `dev descriptions analyze` subcommands in
      exodata-prose, the instrument for everything below:
      - text stats: word/sentence/paragraph counts, article-length
        distributions (mean/median/percentiles), avg sentence length,
        type-token ratio, title stats
      - n-grams 1–5: corpus + document frequency, top-N, sentence-opener
        n-grams (template monotony detector)
      - pattern mining: number/unit-masked sentence templates, trope
        report (known warts with counts + affected systems)
      - metadata aggregation: tokens by stage, attempts, cache hit rates,
        generation dates, fingerprints
      - anomaly detection: length/token outliers, cross-system duplicate
        sentences, numeric-density anomalies
      - lines/json/csv output + `--report` markdown dump
- [ ] B3: rig-core providers — hardcoded profile table (base URL, model,
      key env var; keys stay in `.env`), `--profile`, smaller models;
      generations write `description_<label>.md` with multi-generation
      metadata records; `description.md` stays the served deepseek-flash
      output; promotion manual
- [ ] B4: content improving — fixes chosen from B2 data (year-extreme
      comparisons, 365-day anchor, inventory monotony, length target,
      retry/critic efficiency); baseline-15 then full regeneration;
      record totals
- [ ] B5: deployment (last) — A2 upload channel per roadmap

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
- Content layout: nothing under `content/systems/` is tracked (prose and
  notes are private local data; deployment via the roadmap's upload
  channel). Prompts and the guide at `content/` root stay tracked. User
  handles all Git staging/commits.
- Baseline untrack and gitignore of `content/systems/` completed
  2026-09-10 (user-run).
- Open decisions: mixed-provenance mass ranking (55 Cnc) — current output
  avoids it; article length target; serving/deployment packaging (see the
  roadmap's upload-channel entry); translation pipeline (few weeks out,
  after pass-2 content lands).

## Next

Start B2 (analysis suite) in `exodata-prose`.
