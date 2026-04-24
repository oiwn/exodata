# Testing

## Unit Tests

```bash
# Run all tests
cargo test

# Workspace tests
cargo test --workspace

# Specific crate
cargo test -p exo-cli
cargo test -p exo-core

# With output
cargo test -- --nocapture
```

## E2E Tests (Playwright)

End-to-end tests live in `end2end/tests` and cover 3 smoke flows:

- SSR + hydration on `/stellarhosts`
- SSR + hydration on `/exoplanets`
- `/` → client navigation to `/stellarhosts` (metadata available for column selector)

### One-time setup

```bash
cd end2end
npm ci
npx playwright install chromium
cd ..
```

### Run with one command (recommended)

```bash
cargo leptos end-to-end
```

This builds the app, starts the SSR server, and runs the Playwright suite.

### Run Playwright directly

Start the app server in another terminal first:

```bash
# terminal 1
cargo leptos watch

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

## Code Coverage

Coverage runs in CI via `.github/workflows/coverage.yml` using `cargo-llvm-cov` and uploads to [Codecov](https://codecov.io).

```bash
# Generate coverage locally (requires llvm-tools-preview)
cargo llvm-cov --workspace --lcov --output-path lcov.info

# View report in terminal
cargo llvm-cov --workspace
```

Frontend components (`src/components/`) are excluded from coverage since they require a browser environment.
