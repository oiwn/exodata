# Current Context

## Active Tasks

### Task 1: Proper logging

Add `tracing` instrumentation to the server-side data path to diagnose the SSR streaming failure.
Without logs we can't tell whether the server function is called, completes, panics, or is never invoked.

**Already done**:
- `get_stellarhosts_page` — logs entry (`page`, `columns`) and exit (`total`) via `tracing::info!`

**Still needed**:
- `get_exoplanets_page` — same entry/exit pattern
- `get_stellarhosts_data_cached` / `get_exoplanets_data_cached` — log cache hit vs miss
- Any panic/error paths in server functions should log via `tracing::error!`

**What to look for in production logs after deploy**:
1. Neither log line → server function never called → Leptos SSR not resolving resource
2. Entry logged, exit not → panic or error inside the function body
3. Both logged → function is fine, problem is in Leptos streaming/serialization layer

### Task 2: Lazy Routes (Future / Post-hydration-fix)

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

