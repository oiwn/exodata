# Testing

## Unit Tests

```bash
# Run all tests
cargo test

# Workspace tests
cargo test --workspace

# Specific crate
cargo test -p exodata
cargo test -p exodata-core

# With output
cargo test -- --nocapture
```

## E2E Tests (Playwright)

End-to-end tests live in `end2end/tests` and cover six smoke flows:

- SSR + hydration on `/stellarhosts`
- SSR + hydration on `/exoplanets`
- Table interactions preserve sorting, filtering, column, and pagination state
- Invalid and out-of-range catalog pages return the branded 404 page
- `/` → client navigation to `/stellarhosts` (metadata available for column selector)
- Detail provenance tables stay contained on a mobile viewport

### One-time setup

Use Node.js 24 with npm 11.

```bash
cd end2end
npm ci
npx playwright install chromium
npm run prepare-fixtures
cd ..
```

`prepare-fixtures` stages the small, deterministic Parquet fixtures from
`crates/exo-core/tests/fixtures` under the ignored `end2end/runtime-data`
directory and generates matching metadata TOML files. These fixtures are
stable test material, not a source of truth for live catalog values.

Playwright declares `fsevents` as an optional macOS dependency. npm 11 may ask
whether to approve its native install script; leave it unapproved. The smoke
suite does not use Playwright's file-watching support, and Linux CI does not
install this optional package.

### Run with one command (recommended)

```bash
EXO_DATA_DIR=end2end/runtime-data cargo leptos end-to-end --split
```

This builds the lazy-route WASM chunks, starts the SSR server, and runs the
Playwright suite. The `--split` flag is required because the application uses
Leptos lazy routes.

Type-check the E2E sources independently with:

```bash
cd end2end
npm run typecheck
```

### Run Playwright directly

Start the app server in another terminal first:

```bash
# terminal 1
EXO_DATA_DIR=end2end/runtime-data cargo leptos watch --split

# terminal 2
cd end2end
npx playwright test
```

### HTML report

```bash
cd end2end
npx playwright test --reporter=html
npx playwright show-report
```

`show-report` serves the report locally (default: `http://localhost:9323`) until you stop it with `Ctrl+C`.

### Continuous integration

`.github/workflows/e2e.yml` runs the Chromium smoke suite for pull requests to
`main` and on manual dispatch. It installs Node.js 24, Rust's
`wasm32-unknown-unknown` target, Cargo Leptos, and Chromium's Linux
dependencies, then prepares the same small fixtures used locally.

The tests run against the checked-out code through
`cargo leptos end-to-end --split`. Failed jobs upload the HTML report as the
`playwright-report` artifact.

## Code Coverage

Coverage runs in CI via `.github/workflows/coverage.yml` using `cargo-llvm-cov` and uploads to [Codecov](https://codecov.io).

```bash
# Generate coverage locally (requires llvm-tools-preview)
cargo llvm-cov --workspace --lcov --output-path lcov.info

# View report in terminal
cargo llvm-cov --workspace
```

Frontend components (`src/components/`) are excluded from coverage since they require a browser environment.
