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

### Task 2: Fix SSR 504 on Table Routes

See `specs/ssr-streaming-issue.md` for full details.

**TL;DR**: `/exoplanets` and `/stellarhosts` routes use `SsrMode::Async` which holds the HTTP
connection open until all resources resolve. On a 1-vCPU DO droplet this exceeds Nginx's
`proxy_read_timeout`, causing 504. `spawn_blocking` for Polars is **already implemented**
(common.rs:283-299, 345-361) — that was never the root cause.

**Chosen fix**: Change `ssr=SsrMode::Async` to `ssr=SsrMode::OutOfOrder` in `src/app.rs:91,97`.
This sends the HTML shell immediately and streams resource data in, eliminating the timeout.

**Status**: Implemented (`src/app.rs:91,97`). Needs production verification.

---

### Task 3: Lazy Routes (Future / Post-hydration-fix)

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

---

## Key Files

| File | Purpose |
|------|---------|
| `src/app.rs` | Shell (head/body), App component, route definitions |
| `src/lib.rs` | WASM hydration entry point (`hydrate_body`) |
| `src/server/common.rs` | Polars data logic + cached variants with `spawn_blocking` |
| `src/server/functions.rs` | Leptos server functions (thin wrappers over common.rs) |
| `src/server/handlers.rs` | Axum REST handlers + `ApiState` struct |
| `src/main.rs` | Server startup, data loading, prewarm, Axum router setup |
| `style/tailwind.css` | Tailwind CSS input file |
| `Cargo.toml` | `wasm-release` profile, `hydrate`/`ssr` features |

## References
- Issue #26: https://github.com/oiwn/exoplanets-catalog/issues/26
- Leptos hydration docs: https://book.leptos.dev/ssr/24_hydration_bugs.html
- Leptos lazy routes example: https://github.com/leptos-rs/leptos/tree/main/examples/lazy_routes
