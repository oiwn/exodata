# Current Context

## Active Tasks

### Task 1: Proper logging

Add `tracing` instrumentation to the server-side data path to diagnose the SSR streaming failure.
Without logs we can't tell whether the server function is called, completes, panics, or is never invoked.

**Done**:
- `tracing-subscriber` added with `env-filter`; initialized in `main.rs` (defaults to `info` level, overridable via `RUST_LOG`)
- `get_stellarhosts_page` — entry/exit `info!` + error-path `error!`
- `get_exoplanets_page` — same entry/exit + error-path pattern
- `get_stellarhosts_data_cached` / `get_exoplanets_data_cached` — `debug!` cache hit vs miss
- `get_stellar_host_detail`, `get_planets_for_host`, `get_exoplanet_detail` — entry `info!` + error `error!`
- `main.rs` — `println!` replaced with `tracing::info!`

**What to look for in production logs after deploy**:
1. Neither log line → server function never called → Leptos SSR not resolving resource
2. Entry logged, exit not → panic or error inside the function body
3. Both logged → function is fine, problem is in Leptos streaming/serialization layer

### Task 2: Lazy Routes (Future / Post-hydration-fix)

^^^ need to discuss it

**Concept**: Split WASM bundle by route using `#[lazy]` / `cargo leptos --split`.
Only load WASM for a route when navigated to.

**Candidates** (high value routes):
- `/exoplanets` — heavy table + column selector
- `/stellarhosts` — heavy table + column selector
- `/exoplanets/:pl_name` — detail view
- `/stellarhosts/:hostname` — detail view

**Why defer**: Leptos 0.8 lazy routes have known instabilities (panic on multiple loads,
hydration fallback bugs). Also requires switching `hydrate_body()` → `hydrate_lazy()` in
`lib.rs`, which interacts with the overlay fix. Do after Tasks 1 and 2 are stable.

**Note**: WASM size optimization is already maxed (`opt-level='z'`, `lto=true`,
`codegen-units=1`, `wasm-release` profile). Lazy routes would give the next meaningful
reduction.

