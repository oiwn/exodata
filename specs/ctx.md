# Current Task Context: Upgrade and Harden Rust Dependencies
State: in progress

## Plan

- [x] Inventory manifests, lockfile, build entry points, CI, and current audit findings.
- [x] Update canonical requirements on `dev` in staged breaking-upgrade groups.
- [x] Adapt source code for upgraded dependency APIs while preserving public behavior.
- [x] Add the Cargo audit policy, locked build enforcement, and durable overview documentation.
- [x] Run the fastest meaningful manual checks, then formatting, locked Clippy/tests/build, Playwright smoke tests, audit, and workflow validation.
- [x] Review the final diff and record verification results without archiving the task.

## Findings

- Target stable versions and accepted audit exceptions are defined by the supplied upgrade plan dated 2026-09-05.
- `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` may be ignored only for trusted offline VOTable input and unused Polars cloud-XML paths; informational advisories remain visible.
- Serde `1.0.229` fails because VOTable `0.7.0` imports the removed private `serde::__private228` module; retain exact `1.0.228` until VOTable is compatible.
- Locked compile, CI-scope Clippy, 175 workspace tests, release cargo-leptos build, coverage, and six Playwright smoke tests pass.
- Cargo Audit exits successfully with only the accepted Bincode, Paste, and proc-macro-error2 informational warnings; `h2` is `0.4.19` and `chacha20` is `0.10.2`.

## Context

Preserve routes, REST/MCP contracts, CLI commands, serialization formats, and observable behavior. Keep `Cargo.lock` committed and use locked resolution in builds and CI.

## Next

Await user review and explicit task closure before archiving or updating the changelog.
