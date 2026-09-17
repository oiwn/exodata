---
name: exodata-checks
description: >-
  Select and run focused verification for changes in this Exodata repository:
  Rust CLI/core/backend checks, Leptos SSR and hydration, and Playwright smoke
  flows. Use when implementing, reviewing, or validating repository changes.
---

# Exodata Checks

Run commands from the repository root unless a step says otherwise.
Read [testing coverage](../../../specs/testing.md) and the relevant technical
spec before choosing checks. Use the smallest check that exercises the changed
behavior; documentation-only changes need link/command/diff checks, not a build.

## Choose the Check

| Change | Initial verification |
| --- | --- |
| CLI parsing, config, output, scanner | Relevant test in package `exodata`, then the affected CLI command with controlled local inputs |
| Core transformations or insights | Relevant test in `exodata-core`; fixture counts are sample inputs, not live catalog requirements |
| REST, MCP, server data, shared payloads | Relevant `exodata-web` library tests and the affected local endpoint/tool; check the hydration build if shared payloads changed |
| Table state or frontend behavior | Relevant state tests, split SSR/hydration build, then direct load and client navigation through the affected flow |
| Styling | Split build, then desktop/mobile inspection of the affected component |
| Docs, instructions, ignore rules | Local paths/anchors, commands against definitions, ignore behavior, and `git diff --check` |

For a regression, reproduce the reported behavior where practical. After
compilation, do the fastest meaningful manual check before broader test loops.
Fix an observed failure directly. If the user will perform UI verification,
hand them the exact startup command, URL, and relevant actions instead of
continuing broad automated loops.

## Rust Commands

Package names differ from some directories/import names:
`exodata` (CLI), `exodata-core`, `exodata-types`, and `exodata-web`.

Examples; select the relevant test/filter rather than running every command:

```bash
cargo test --locked -p exodata --test descriptions_tests
cargo test --locked -p exodata-core
cargo test --locked -p exodata-web --lib
cargo run --locked -p exodata -- --help
```

When Rust formatting is needed, run `cargo fmt --all`. Broader checks, when
the change or CI scope requires them:

```bash
cargo test --locked --workspace
cargo clippy --locked --all-features --workspace -- -D warnings
```

The Clippy command above matches CI. `just clippy` instead uses
`--all-targets --all-features` and includes targets such as examples; report
which scope failed rather than treating the commands as interchangeable.

For dependency changes, inspect manifests, `Cargo.lock`, the compatibility
policy in [overview.md](../../../specs/overview.md#dependency-policy), and
`.cargo/audit.toml`. Keep version/advisory decisions there, not in this skill.
Coverage commands and exclusions come from
[coverage.yml](../../../.github/workflows/coverage.yml); do not run coverage
for every change.

## Web Setup and Verification

Prerequisites follow [e2e.yml](../../../.github/workflows/e2e.yml): Rust's
`wasm32-unknown-unknown` target, Cargo Leptos, Tailwind, Node.js 24/npm, and
Chromium. Inspect installed tooling before installing anything.

When E2E dependencies or fixtures are missing, from `end2end/`:

```bash
npm ci
npx playwright install chromium
npm run prepare-fixtures
```

The fixture step writes only the ignored E2E runtime directory. Do not refresh
NASA data merely to run a browser smoke test.

From the repository root, run the fixture-backed app:

```bash
EXO_DATA_DIR=end2end/runtime-data cargo leptos watch --split
```

Open `http://127.0.0.1:3000`. Lazy routes require `--split`. Use an available
browser tool or give the user the URL and exact checks. Confirm the overlay
clears, then exercise the affected behavior. For table changes, compare direct
loading with client navigation and preserved sort/filter/columns/page state.
For locale changes, use the [locale contract](../../../specs/localization.md).
For styling, inspect mobile overflow as well as desktop layout.

To build without starting the app: `cargo leptos build --split`.
For release-build changes: `cargo leptos build --release --split`.

With the app running, execute the selected Playwright test from `end2end/`,
for example:

```bash
npx playwright test --grep "metadata is available" --reporter=html
npm run typecheck
```

`PLAYWRIGHT_BASE_URL` overrides the default URL. The complete suite can build
and start its own app from the repository root:

```bash
EXO_DATA_DIR=end2end/runtime-data cargo leptos end-to-end --split
```

Do not start a second server on the same port. The selected test command above
creates an HTML report. When a test fails,
inspect it with `npx playwright show-report` from `end2end/`; do not rerun the
whole suite merely to generate a report.

## API and Completion Checks

Trace affected REST, Leptos, MCP, and CLI paths through their shared data
functions; verify only the interfaces changed. Use local fixture/current data
appropriate to the test. Inspect schema before guessing SQL columns. Public
REST examples are in [docs/api.md](../../../docs/api.md), and MCP contracts are
in [docs/mcp.md](../../../docs/mcp.md).

Finish with `git diff --check` and inspect the scoped diff, including new files.
Report commands actually run, manual results, and any remaining limitation.
Do not report historical test results as fresh verification.
