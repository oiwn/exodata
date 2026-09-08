# Testing

This specification describes verification coverage and fixtures. Setup,
commands, and selection of focused checks live in the
[exodata-checks skill](../.agents/skills/exodata-checks/SKILL.md).

## Rust Coverage

- Core tests exercise fixture loading and table aggregations.
- CLI tests cover command help/grouping, configuration precedence, output
  conversion, skill installation, and read-only description scanning.
  Description preparation tests cover selection failures, qualifiers and mass
  provenance, conversions/comparison suppression, deterministic artifacts,
  identifier collisions, and replacement behavior preserving articles/metadata.
  Paired-write tests inject installation failures to verify restoration of
  previous files or absences and preservation of pending recovery backups.
  API/local backend and download changes need focused verification; the presence
  of those modules does not establish automated coverage.
- Web tests cover data queries, summaries, exports, REST/MCP, sitemaps, locale
  helpers, table state, and Markdown rendering.
- The expected behavior comes from current contracts and implementation;
  fixtures provide test inputs, not authoritative live catalog values.

CI uses locked workspace tests in
[tests.yml](../.github/workflows/tests.yml). Formatting, Clippy, and typo
checks are configured in [code-quality.yml](../.github/workflows/code-quality.yml).
Local Justfile Clippy and CI Clippy have different target scopes; the checks
skill explains when to use each.

## Browser Smoke Coverage

[end2end/tests/smoke.spec.ts](../end2end/tests/smoke.spec.ts) covers:

- SSR and hydration on both catalog tables.
- Sorting, filtering, column selection, and pagination query-state preservation.
- Branded 404 responses for invalid and out-of-range catalog pages.
- Metadata availability after homepage-to-stellarhosts client navigation.
- Host and planet provenance containment on mobile.

The suite uses Chromium with one worker. Its base URL defaults to
`http://127.0.0.1:3000` and can be overridden with `PLAYWRIGHT_BASE_URL`.
[playwright.config.ts](../end2end/playwright.config.ts) is the source for
timeouts, retries, and reporting.

## Fixtures

[prepare-fixtures.mjs](../end2end/prepare-fixtures.mjs) copies stable Parquet
fixtures from `crates/exo-core/tests/fixtures` into ignored
`end2end/runtime-data` and generates matching metadata TOML files.
The server selects them with `EXO_DATA_DIR`. Fixture preparation does not
require downloading NASA exports.

[e2e.yml](../.github/workflows/e2e.yml) installs the required Rust/WASM, Cargo
Leptos, Tailwind, Node, and Chromium tooling, prepares these fixtures, and runs
the split Leptos build and smoke suite. Failed jobs upload the HTML report.

## Coverage Reports

[coverage.yml](../.github/workflows/coverage.yml) runs locked workspace
`cargo-llvm-cov` and uploads LCOV to Codecov. Its filename filter excludes
`src/bin/.*`, `src/components/.*`, `src/app.rs`, `src/metadata.rs`, and
`src/error_template.rs`. Rust coverage therefore does not replace browser
verification of UI behavior.

Report test results for the actual run and scope. Historical test counts or
past build successes are not evidence for a new change.
