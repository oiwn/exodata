# Code Review Findings

Review conducted on 2025-02-12 to identify sloppy code, duplicates, dead code, and outdated documentation.

---

## 1. Major Duplicates

### 1.1 Table Page Components (~540 lines each)

**Files:**
- `src/components/exoplanets_table.rs` (541 lines)
- `src/components/stellarhosts_table.rs` (539 lines)

**Issue:**
These two components are nearly identical (~1080 lines total) with only the following differences:
- Default columns list
- Server function called (`get_exoplanets_page` vs `get_stellarhosts_page`)
- URL paths (`/exoplanets` vs `/stellarhosts`)
- Header titles and emoji (`🪐 Exoplanets Catalog` vs `⭐ Stellar Hosts Catalog`)
- Link column name (`pl_name` vs `hostname`)
- Link base path (`/exoplanets/` vs `/stellarhosts/`)

**Suggestion:**
Create a generic table page component that accepts configuration:
```rust
#[component]
pub fn GenericTablePage(
    page_type: PageType,  // Exoplanets or StellarHosts
    title: &'static str,
    emoji: &'static str,
    default_columns: Vec<String>,
    data_fetcher: impl Fn(...) + Clone,
    link_column: String,
    link_base: String,
) -> impl IntoView
```

**Estimated savings:** ~900 lines of code (keeping only one generic implementation)

---

## 2. Unused / Dead Code

### 2.1 Unused Component: BuyMeCoffee

**File:** `src/components/buy_me_coffee.rs`

**Issue:**
The `BuyMeACoffee` component is defined but commented out in `src/components/mod.rs`:
```rust
// pub mod buy_me_coffee;
```

The component exists and is complete but is never used in the application.

**Recommendation:**
Either enable the component and use it, or delete the file and the commented line.

---

### 2.2 Legacy Server Function

**File:** `src/stellarhosts.rs`

**Issue:**
This file contains a legacy server function `get_exoplanet_data()` that:
- Loads data from VOTable files (`data/stellarhosts.vot`)
- Does manual JSON serialization
- Returns raw string data
- Is not called from anywhere in the current codebase

The current implementation uses:
- Parquet files for data storage
- Common business logic in `src/server/common.rs`
- Structured data types

**Recommendation:**
Delete `src/stellarhosts.rs` entirely. It's legacy code from an earlier implementation.

---

## 3. Clippy Warnings

Running `cargo clippy --all-targets` revealed multiple code quality issues:

## 4. Known Technical Debt

### 4.1 Duplicate Type Definition

**File:** `src/server/functions.rs`

```rust
// NOTE: This is a temporary duplicate of exo_core::metadata::ColumnMetadata
// to avoid bringing exo-core dependencies into the client WASM bundle.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub datatype: String,
}
```

The comment explicitly acknowledges this is a temporary workaround. This creates:
- Two definitions of the same type
- Manual conversion function (`impl From<exo_core::metadata::ColumnMetadata>`)
- Potential for divergence

**Recommendation:**
Use feature flags on `exo-core` to exclude heavy dependencies when compiling to WASM:
```toml
# In exo-core/Cargo.toml
[features]
default = []
wasm-friendly = []  # Excludes Polars and other heavy deps
```

### 4.2 TODO Comments

**Locations:**
1. `src/cli.rs:29` - `// TODO: setup logging depending on verbosity level`
2. `crates/exo-core/src/tables/stellarhosts.rs:4` - `// TODO: Add stellar host specific domain logic here:`
3. `crates/exo-core/src/tables/exoplanets.rs:4` - `// TODO: Add exoplanet specific domain logic here:`

---

## 5. Hardcoded Values

### 5.1 Column Names in Table Component

**File:** `src/table/table.rs:289-297`

```rust
fn format_column_name(col: &str) -> String {
    match col {
        "hostname" => "Star Name".to_string(),
        "sy_dist" => "Distance (pc)".to_string(),
        "st_teff" => "Temperature (K)".to_string(),
        "st_mass" => "Mass (M☉)".to_string(),
        "sy_pnum" => "Planets".to_string(),
        _ => col.to_string(),
    }
}
```

**Issue:**
Column display names are hardcoded. This should use metadata from the backend to:
- Support dynamic column selection
- Allow for future column additions without code changes
- Keep display logic centralized

---

## 6. Outdated Documentation

### 6.1 README_LEPTOS.md

**Issue:**
This file is the default Leptos template README and has not been updated for the exoplanets catalog project. It contains:
- Generic template instructions
- References to `start-axum` template
- Unchanged licensing text
- No mention of project-specific features

**Recommendation:**
Delete this file. The main `README.md` contains all relevant documentation.

---

### 6.2 specs/web-backend.md Outdated API Routes

**Issue:**
The spec documents API routes at `/api/*`:
```
GET /api/stellarhosts
GET /api/exoplanets
```

**Current Implementation:**
Routes are at `/rest/*` (per `src/main.rs:133` and `handlers.rs:171-181`):
```rust
.nest_service("/rest", server::api_routes(api_state))
```

**Also Missing:**
- SQL query endpoint (`/rest/query`) - added but not documented
- Swagger UI (`/swagger-ui`) - added but not documented
- Caching implementation - exists in `cache.rs` but not mentioned in spec

### 6.3 specs/web-backend.md Outdated Response Structures

**Issue:**
Spec documents `ApiResponse<T>` with:
```rust
pub struct ApiResponse<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub limit: usize,
    pub filters: QueryParams,  // This field doesn't exist in current implementation
}
```

**Current Implementation:**
From `src/server/handlers.rs:64-84`:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse {
    pub data: Vec<Value>,
    pub total: usize,
    pub total_all: usize,  // This field exists but isn't documented
    pub page: usize,
    pub limit: usize,
    pub columns: Vec<String>,  // This field exists but isn't documented
}
```

### 6.4 specs/web-backend.md Missing Query Parameters

**Current Implementation:**
From `src/server/handlers.rs:41-61`:
```rust
pub struct QueryParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub sort_by: Option<String>,
    pub order: Option<String>,
    pub columns: Option<String>,
    pub filter: Option<String>,  // Text filter on first column - not documented
}
```

**Spec:**
Documents many range filters (`sy_dist_min`, `st_teff_max`, etc.) that don't exist in the current implementation.

### 6.5 specs/data-management.md References Non-existent Scripts

**Issue:**
The spec describes shell scripts in a `scripts/` directory:
- `scripts/fetch-data.sh`
- `scripts/convert-latest.sh`
- `scripts/update-data.sh`
- `scripts/cleanup-old.sh`

**Current State:**
No `scripts/` directory exists in the project. Data management appears to be done manually or via the CLI tool `exo-cli`.

**Recommendation:**
Either implement the described scripts or update the documentation to reflect the actual data management workflow.

---

## 7. Potential Improvements

### 7.1 Consolidate Query String Building

**Files:**
- `src/table/table.rs:260-286` (function `build_table_query`)
- Similar logic in table page components

Query string building logic appears in multiple places and could be consolidated.

### 7.2 Shared Type Conversions

The codebase has multiple places converting between similar types (e.g., `exo_core::metadata::ColumnMetadata` vs local `ColumnMetadata`). A shared type conversion module could reduce duplication.

---

## Summary

| Category | Count | Impact |
|----------|--------|--------|
| Major duplicate code blocks | 2 | High (1080 lines) |
| Unused/dead files | 2 | Medium |
| Clippy warnings | 24+ | Low-Medium |
| TODO comments | 5 | Low |
| Outdated spec sections | 4 | Medium |
| Hardcoded values | 1 | Low |

**Priority Actions:**
1. Refactor table page components into single generic component (saves ~900 lines)
2. Delete unused `src/stellarhosts.rs` and `src/components/buy_me_coffee.rs`
3. Update `specs/web-backend.md` to reflect current API structure
4. Fix high-impact Clippy warnings (collapsible if statements, unnecessary patterns)
5. Update or delete `README_LEPTOS.md`
6. Address ColumnMetadata duplication via feature flags in exo-core
