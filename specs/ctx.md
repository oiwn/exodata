# Current Context

## Active Tasks

### Task 1: Fix Hydration Overlay (Issue #26) ✓ DONE

**Solution**: `pre-hydration` class added to `<html>` via inline script before body parses.
CSS blocks interaction and shows overlay+spinner via `body::before` / `body::after`.
Class removed by WASM after `hydrate_body()` completes.

**What didn't work during implementation**:
- `html::before` pseudo-element — doesn't render; use `html.pre-hydration body::before` instead
- `html.pre-hydration *` cursor/pointer-events works fine

**Files changed**:
- `src/app.rs` — inline `<script>` in `shell()` head
- `src/lib.rs` — `web_sys` class removal after hydration
- `style/tailwind.css` — overlay + spinner via `body::before` / `body::after`
- `Cargo.toml` — `web-sys` added with features, scoped to `hydrate`

---

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
