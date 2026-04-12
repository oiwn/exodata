# Current Context

## Detail Page Refactor

### Scope

- First: refactor `src/components/stellarhost_detail/` to semantic feature CSS.
- Next: refactor `src/components/exoplanet_detail.rs` to match the current architectural shape of `stellarhost_detail`.

### Current Situation

- `stellarhost_detail` is already structurally split into feature files such as:
  - `page.rs`
  - `hero.rs`
  - `summary.rs`
  - `planets.rs`
  - `provenance.rs`
- `stellarhost_detail` still carries large inline Tailwind class strings in Rust and should be moved to semantic feature CSS.
- `src/components/exoplanet_detail.rs` is still a large single-file component mixing:
  - route/page shell
  - suspense/loading/error handling
  - summary cards
  - records section
  - record card/table rendering
  - formatting logic

### Goals

- Reduce inline class noise in detail-page Rust code.
- Keep the current visual design direction, but express it through semantic feature styles.
- Make `exoplanet_detail` structurally consistent with `stellarhost_detail`.
- Make `exoplanet_detail` feel visually related to `stellarhost_detail` without forcing identical content structure.

### Specification

#### 1. Stellar Host Detail Styling Cleanup

- Add `style/components/stellarhost-detail.css`.
- Import it from `style/tailwind.css`.
- Move large section-level class groups into semantic feature classes.
- Keep small one-off utility classes inline only when they improve readability.
- Do not change behavior or data flow in this pass.

#### 2. Exoplanet Detail Structural Refactor

- Convert `src/components/exoplanet_detail.rs` into a feature module:
  - `src/components/exoplanet_detail/mod.rs`
  - `page.rs`
  - `hero.rs`
  - `summary.rs`
  - `records.rs`
  - optional shared `format.rs`
- Separate page/meta/resource handling from section rendering.
- Extract loading and error states into small page-level components or helpers.
- Keep route exports stable so the app shell and routing do not need broader changes.

#### 3. Exoplanet Detail Visual Alignment

- Add `style/components/exoplanet-detail.css`.
- Import it from `style/tailwind.css`.
- Move repeated page-level and section-level Tailwind class piles into semantic classes.
- Align `exoplanet_detail` with the same design family as `stellarhost_detail`, especially for:
  - page shell
  - back link
  - hero/header treatment
  - section spacing/rhythm
  - loading state
  - error state
- Keep exoplanet-specific content and section composition appropriate to the page’s actual data model.

### Non-Goals

- No server/data contract changes.
- No new styling library or scoped styling tool.
- No forced one-to-one section parity when the exoplanet page needs different content structure.

## Shared Table State Follow-Up

### Goal

- Reduce signal sprawl in `src/components/stellarhosts_table/page.rs`.
- Shape the refactor so the same approach can later be reused in `src/components/exoplanets_table.rs`.

### Current Observation

- `stellarhosts_table/page.rs` still declares many adjacent signals for one logical area of state.
- Most of these belong to table/query behavior rather than page-specific UI.
- `exoplanets_table.rs` still has the older, larger version of the same pattern.

### Proposed Direction

- Extract shared table/query signal state into a reusable struct in `src/table/`.
- Keep transient UI-only state separate from query state.

### Likely Split

- Shared table/query state:
  - `current_page`
  - `sort_column`
  - `sort_order`
  - `selected_columns`
  - `filter_text`
  - possibly `filter_input`
- Local UI state:
  - `selector_is_open`
  - `is_loading`
  - `has_loaded`

### Design Intent

- The shared struct should store Leptos signals, not plain snapshot values.
- It should expose small helper methods so it is more useful than a passive field bag.
- It should be designed for reuse by both table pages, not only `stellarhosts_table`.

### Suggested Refactor Phases

#### Phase A: Shared Query Signal Struct

- Add a shared struct in `src/table/` for table/query signals.
- Add helpers such as:
  - current query snapshot
  - set page
  - set filter
  - set sort for column
- Migrate `stellarhosts_table/page.rs` onto that struct first.
- Status: next

#### Phase B: Shared Pagination View State

- Add a shared pagination view-state struct in `src/table/`.
- Use a generic name such as `TablePaginationState` or `PaginationViewState`.
- Target repeated pagination props such as:
  - `start`
  - `end`
  - `total`
  - `current_page`
  - `total_pages`
  - `can_go_prev`
  - `can_go_next`
- Migrate `stellarhosts_table` first, then reuse the same struct in `exoplanets_table`.
- Keep callbacks and optional page links explicit unless bundling them clearly improves readability.
- Status: completed

#### Phase C: Reuse In Exoplanets Table

- Apply the same shared table/query signal struct to `src/components/exoplanets_table.rs`.
- Compare whether more shared controller logic can be extracted after both pages use the same state model.
