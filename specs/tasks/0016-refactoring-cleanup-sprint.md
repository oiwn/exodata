---
id: 0016-refactoring-cleanup-sprint
status: draft
scope: []
created: 2026-10-29
---
# Task: refactoring cleanup sprint

## Plan

- [ ] Small bugs first: reproduce and fix #27 (cache prewarm key mismatch), #144 (progress broken in VOTable → Parquet conversion), #36 (stale wasm served from cache).
- [ ] #157: evaluate a faster linker (mold/lld on Linux, default/lld on macOS) and record build-time before/after.
- [ ] #129: replace footer utility-class markup with semantic component classes in `style/components/`.
- [ ] #111: remove MCP mentions from the `exodata` CLI package metadata and help text.
- [ ] Roadmap "Localized Route Declarations": declare page routes once for English, `/zh-CN`, `/ja`.
- [ ] Roadmap "Shared Measurement Selection": finish shared selection for detail pages and exports, and measurement qualifiers.
- [ ] Roadmap "Update tailwind": bump the pinned Tailwind version and verify styles.
- [ ] Split large files into child modules (`name.rs` + `name/`): `crates/exo-prose/src/descriptions/prepare.rs`, `validate.rs`, `crates/exo-core/src/tables/overview.rs`, `crates/exo-cli/src/main.rs`. Add user-identified refactor targets to [cleanup.md](../cleanup.md) before starting.

## Acceptance

- [ ] Each item ships as its own commit-sized change with scope declared through specdev; this task may be split into child tasks.
- [ ] Bug fixes have a reproducing test or documented reproduction before the fix.
- [ ] Refactors preserve behavior: public URLs, REST/MCP contracts, CLI output, and route locale behavior unchanged unless explicitly approved.
- [ ] `specs/cleanup.md` lists refactor targets with file and 1–3 sentence rationale; resolved entries are removed.
- [ ] `cargo fmt --all -- --check`, `cargo leptos build --split`, `cargo test --locked --workspace --all-features`, and `cargo lx --locked -- -D warnings` pass.

## Context

- Candidates collected from open GitHub issues and [roadmap.md](../roadmap.md) on 2026-10-27. Issue numbers identify requests, not verified current state; re-check each issue body before work.
- The user will review the code and add further refactor targets; ordering and grouping are confirmed with the user before advancing from draft.
- Infrastructure work (tarball deploy, Ferron, monitoring) is tracked separately in [infrastructure.md](../infrastructure.md).

## Log

- 2026-10-29 created: refactoring cleanup sprint
