# Current Context

## Active Task: Reduce WASM Bundle via Code Splitting

### Goal

Reduce the **1.2 MB** (release, `wasm-release` profile) WASM bundle so pages load faster. Compiler-level size optimization is already maxed (`opt-level='z'`, `lto=true`,`codegen-units=1`). The next meaningful reduction requires code splitting via lazy routes.

### Approach: Lazy Routes (`#[lazy_route]` + `cargo leptos --split`)

Lazy routes split the WASM binary into separate chunks per route. Each chunk loads only when the user navigates to that route. Data loading and view rendering happen concurrently (no waterfall).

Official example: <https://github.com/leptos-rs/leptos/tree/main/examples/lazy_routes>

---

### Key Patterns (from official example)

**Route declaration** — wrap lazy views with `Lazy::<Name>::new()`:

```rust
use leptos_router::{Lazy, LazyRoute, lazy_route};

<Routes fallback=|| "Not found.">
    <Route path=StaticSegment("") view=ViewA/>                        // eager
    <Route path=StaticSegment("c") view={Lazy::<ViewC>::new()}/>      // lazy
    <ParentRoute path=StaticSegment("d") view={Lazy::<ViewD>::new()}> // lazy parent
        <Route path=StaticSegment("") view={Lazy::<ViewE>::new()}/>   // lazy child
    </ParentRoute>
</Routes>
```

**Lazy route struct** — `LazyRoute` trait with `data()` + `view()`:

```rust
#[derive(Clone)]
struct ViewC {
    data: LocalResource<Vec<Album>>,
}

#[lazy_route]
impl LazyRoute for ViewC {
    fn data() -> Self {
        // Synchronous: create Resources, signals, etc.
        Self { data: LocalResource::new(lazy_data) }
    }

    fn view(this: Self) -> AnyView {
        // This body is code-split into its own WASM chunk
        view! {
            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                // render data
            </Suspense>
        }.into_any()
    }
}
```

**Server functions** — no changes needed. `#[lazy]` on server functions was considered but rejected: the savings are negligible (client stubs are tiny vs view code), and the `#[lazy]` + `#[server]` macro interaction adds unnecessary compile-time complexity.

### Required Changes

**`src/lib.rs`**: `hydrate_body(App)` → `hydrate_lazy(App)`

```rust
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_lazy(App);
    let _ = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .map(|el| el.class_list().remove_1("pre-hydration").ok());
}
```

**Build command**: `cargo leptos build --split`

---

### Implementation Plan

#### Phase 1: Prove pipeline with `/about` + `/overview` (simplest routes, static, ~416 lines combined)

Lazy routes are per-`<Route>` — each `Lazy::<X>::new()` produces its own `.wasm` chunk, so routes can't be merged. Both `/about` (182 lines, fully static) and `/overview` (234 lines, 1 server fn `get_stats`) are trivial to convert and validate the pipeline.

1. Change `src/lib.rs`: `hydrate_body(App)` → `hydrate_lazy(App)`
2. In `src/components/about.rs`, add alongside existing `AboutPage`:

```rust
#[derive(Clone)]
pub struct AboutLazy;

#[lazy_route]
impl LazyRoute for AboutLazy {
    fn data() -> Self { Self }
    fn view(this: Self) -> AnyView {
        view! { <AboutPage/> }.into_any()
    }
}
```

3. In `src/components/overview.rs`, add alongside existing `OverviewPage`:

```rust
#[derive(Clone)]
pub struct OverviewLazy;

#[lazy_route]
impl LazyRoute for OverviewLazy {
    fn data() -> Self { Self }
    fn view(this: Self) -> AnyView {
        view! { <OverviewPage/> }.into_any()
    }
}
```

4. Update `src/app.rs`:
   - `<Route path=StaticSegment("about") view={Lazy::<AboutLazy>::new()}/>`
   - `<Route path=StaticSegment("") view={Lazy::<OverviewLazy>::new()}/>`
5. `cargo leptos build --split` — verify multiple `.wasm` files, all routes work, overlay still disappears

**Checkpoint**: if this fails, debug before proceeding.

#### Phase 2: Convert heavy table routes (highest value — ~1050 lines split out)

Refactoring pattern for each table component:
1. Extract Resource creation + reactive dependencies into `LazyRoute` struct fields
2. Move `view!` body into `fn view(this: Self) -> AnyView`
3. Keep original `#[component]` (used by SSR) or remove if fully replaced

**2.1** `StellarHostsTableLazy` — `stellarhosts_table.rs` (528 lines, 1 Resource, 9 signals)

```rust
#[derive(Clone)]
pub struct StellarHostsTableLazy {
    table_resource: Resource<TableQueryState, Result<TableData, ServerFnError>>,
    current_page: RwSignal<usize>,
    sort_column: RwSignal<Option<String>>,
    sort_order: RwSignal<String>,
    selected_columns: RwSignal<Vec<String>>,
    filter_text: RwSignal<String>,
    // ... other signals
}
```

**2.2** `ExoplanetsTableLazy` — same pattern for `exoplanets_table.rs` (530 lines)

**2.3** Update `src/app.rs`:

```rust
<Route path=StaticSegment("stellarhosts")
    view={Lazy::<StellarHostsTableLazy>::new()} ssr=SsrMode::OutOfOrder/>
<Route path=StaticSegment("exoplanets")
    view={Lazy::<ExoplanetsTableLazy>::new()} ssr=SsrMode::OutOfOrder/>
```

#### Phase 3: Convert detail routes (~760 lines)

**3.1** `StellarHostDetailLazy` — `stellarhost_detail.rs` (444 lines, 2 Resources)
**3.2** `ExoplanetDetailLazy` — `exoplanet_detail.rs` (316 lines, 1 Resource)
**3.3** Update `src/app.rs` routes

#### Phase 4: Deploy

- Update `DEPLOY.md` build command to include `--split`
- Verify on production droplet
- Measure final WASM sizes
- **Chunk serving verification** — `cargo leptos build --split` outputs multiple `.wasm` + `.js` files into `target/site/pkg/`. The current Nginx config serves all files from that directory, so no config changes should be needed. Verify after first `--split` build:
  1. HTML shell's `<script>` tags reference the correct chunk JS files
  2. Nginx doesn't have a whitelist that only serves `exoplanets-catalog.wasm`
  3. Content-Type headers are correct for all `.wasm` files
- Inspect `target/site/` for the full file list and update DEPLOY.md with any Nginx changes needed

---

### Affected Files

| File | Change |
|------|--------|
| `src/lib.rs` | `hydrate_body` → `hydrate_lazy` |
| `src/app.rs` | Route declarations + lazy imports |
| `src/components/about.rs` | Add `AboutLazy` + `#[lazy_route]` |
| `src/components/overview.rs` | Add `OverviewLazy` + `#[lazy_route]` |
| `src/components/stellarhosts_table.rs` | Add `StellarHostsTableLazy` + `#[lazy_route]` |
| `src/components/exoplanets_table.rs` | Add `ExoplanetsTableLazy` + `#[lazy_route]` |
| `src/components/stellarhost_detail.rs` | Add `StellarHostDetailLazy` + `#[lazy_route]` |
| `src/components/exoplanet_detail.rs` | Add `ExoplanetDetailLazy` + `#[lazy_route]` |
| `DEPLOY.md` | Add `--split` to build command + chunk serving notes |

### Interaction with Existing Features

- **Pre-hydration overlay**: `hydrate_lazy()` triggers same lifecycle as `hydrate_body()`. Verify overlay disappears correctly in Phase 1.
- **SSR mode**: `SsrMode::OutOfOrder` stays. Compatible with lazy routes.
- **Cache prewarming** in `main.rs`: unaffected. Server-side logic unchanged.
- **Metadata hydration** (JSON in `<head>`): unaffected. Context provided before routes render.
- **URL state sync**: `use_query_map()` / `use_navigate()` work inside `LazyRoute::data()` —
  Router context is available at that point.

### Risks

- **Leptos 0.8 lazy routes relatively new** (summer 2025). Edge cases possible.
- **`hydrate_lazy` + overlay**: was tested with `hydrate_body`. Phase 1 verifies.
- **Chunk loading delay**: first navigation to lazy route fetches chunk. Suspense fallback
  renders during load — match existing loading overlay design.
- **Signal ownership**: signals as struct fields (in `data()`) vs component body. Verify signal
  disposal on navigation away from lazy route.
- **Testing**: server-side tests don't exercise lazy chunk loading. Playwright e2e needed.

### WASM Size Tracking

| Stage | Main Bundle | Chunks | Total |
|-------|:-----------:|:------:|:-----:|
| Baseline | ~1.2 MB | — | ~1.2 MB |
| After Phase 1 (about + overview) | 1.1 MB | 110 KB | ~1.21 MB |
| After Phase 2 (tables) | 803 KB | 365 KB | ~1.17 MB |
| After Phase 3 (details) | 535 KB | 780 KB | ~1.31 MB |

**Result**: 55% reduction in initial page load (1.2 MB → 535 KB). Chunks load on demand.

### Quick Reference

- Lazy routes example: <https://github.com/leptos-rs/leptos/tree/main/examples/lazy_routes>
- Leptos binary size docs: <https://book.leptos.dev/deployment/binary_size.html>
- SSR streaming issue: `specs/ssr-streaming-issue.md`
