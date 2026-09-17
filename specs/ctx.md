# Current Task Context: Generated System Descriptions (#116)

State: completed locally — v10 full catalog generated, finalized, and
verified locally. Production deployment is the remaining manual step
(see Next).

## Outcome (2026-09-17)

- `v10-tuned` accepted by the user as the final generation iteration;
  no further model/prompt/gate iterations are planned.
- Full unlabeled run over all prepared systems: 4,768/4,768 generated,
  0 failed, 0 missing, 7,091,608 recorded tokens, fingerprint v10.
  Stats snapshot: `content/stats/v10.json`. Requests were re-prepared
  with `prepare --all --force` first; `2MASS J11011926-7732383`
  remains unpreparable (no usable stellar-host summary row) and is
  expected to fail `--all` preparation with a nonzero exit.
- Corpus quality verified: 6 ordinary `about` uses corpus-wide, zero
  Earth-year comparisons, zero raw `M sin i`, unbolded-measurement
  anomalies triaged as detector artifacts (object-name fragments,
  compound-adjective recaps) plus structural duplicate count/transit
  sentences that are accepted for a catalog.
- Post-batch copy-edits applied: `an mean density` grammar in 8
  articles; thousands-split bold spans repaired corpus-wide (see
  CHANGELOG 2026-09-17). "is estimated to be" phrasing (~550 articles)
  reviewed and accepted as-is.
- Deployment mechanism changed: descriptions now ship via
  `just ansible-upload-descriptions` + a read-only volume mount at
  `/app/content/systems` instead of being baked into the Docker image
  (which also unbreaks GitHub Actions builds from a clean checkout).
  Details in [DEPLOY.md](../DEPLOY.md).

## Next

1. User: bump version to 0.4.0 in `Cargo.toml`, commit, push; wait for
   the GitHub Actions image build.
2. `just ansible-upload-descriptions` (uploads ~6 MB of
   `description.md` files; needs rsync locally and on the server).
3. `just ansible-deploy`, then verify with `just ansible-status` and
   `just ansible-logs` (expect `Loaded 4768 stellar-host
   descriptions`) and spot-check host pages (for example Kepler-18,
   TOI-125, LHS 1140) on https://exodata.space.
4. After shipping, this file can be reduced to a stub or repurposed
   for the next task.

## Notes

- Local verification already done: full workspace tests, `cargo lx`,
  fmt, typos, and SSR smoke on `/stellarhosts/<host>` pages confirmed
  the finalized corpus renders (bolded comma numbers, grammar fixes).
- Old corpora and experiment variants were moved out by the user;
   `description_pass2.md` variants are no longer on disk. Treated as a
   first generation: no comparisons against previous passes were kept.
- The backup of the pre-v10 content tree is with the user.
