## 2. Overview Page - Data Aggregations Interface

This document outlines the implementation specification for building a single-page web interface that displays key aggregations and statistics about the exoplanet catalog. This task establishes the foundation for the web application architecture.

### 2.1. Goal

Create a minimal, fast-loading overview page that presents key statistics and aggregations from the stellar hosts and exoplanets datasets, demonstrating the catalog's scope and content without complex UI features.

### 2.2. Architecture Principles

#### 2.2.1. Data Layer
- **In-Memory Catalog**: The server loads full Polars DataFrames from parquet files at startup (~25 MB total)
- **Shared State**: A single `AppState` containing the catalog is shared across all access layers
- **Domain Logic Separation**: All data operations (filtering, aggregation, statistics) live in the `tables` module

**Available Aggregations** (found in `src/tables/aggregation.rs`):
1. `temperature_distribution()` - Temperature histogram bins (3000K-10000K)
2. `discovery_timeline()` - Discovery data organized by decade
3. `catalog_crossmatch()` - Catalog coverage statistics (HD, HIP, TIC, GAIA)
4. `photometric_statistics()` - Magnitude distributions across multiple bands

**Simple aggregations to implement for overview page** (4 total):
1. **Total Counts**: Stellarhosts count + Exoplanets count
2. **Average Temperature**: Mean stellar effective temperature (st_teff)
3. **Average Distance**: Mean distance to stellar systems (sy_dist)
4. **Discovery Methods Distribution**: Top 10 discovery methods with counts

#### 2.2.2. Access Layers
The application will have **two independent interfaces** to the same in-memory catalog:

1. **Axum REST API** (External) - **DEFER TO LATER**
   - For 3rd-party consumers and future external integrations
   - Already exists at `/api/stellarhosts` and `/api/exoplanets`
   - Will share the same aggregation functions from `tables` module
   - **NOTE**: Skip for now, focus on Leptos server functions first

2. **Leptos Server Functions** (Internal) - **FOCUS HERE**
   - Magic functions that can run on both client and server
   - During SSR: called directly in-process (fast)
   - After hydration: called via HTTP (automatic)
   - We write functions in `tables` module, then call them from Leptos server functions
   - These will be the same functions that REST API will use later

#### 2.2.3. Key Architectural Decisions
- **Leptos server functions** are the primary focus for Task 2
- Both Leptos and REST API will call the same logic from `tables` module
- Frontend uses server functions exclusively
- Axum routes already configured: Leptos routes handle `/`, `/overview`, etc.; API routes at `/api/*` (see `main.rs:140`)

### 2.3. Scope of Task 2

#### 2.3.1. Backend Components
1. **Tables Module** (`src/tables/`) - **EXISTS**
   - Data loading: `stellarhosts.rs` and `exoplanets.rs` with `load_data()` functions
   - Aggregation functions: `aggregation.rs` has temperature_distribution, discovery_timeline, etc.
   - **TODO**: Add simple aggregation functions for overview stats (4 functions listed above)

2. **Application State** - **EXISTS** (`src/server/handlers.rs:18`)
   - `ApiState` struct holds `Arc<DataFrame>` for both stellarhosts and exoplanets
   - Already loaded at startup in `api_routes()` function
   - Read-only access (Arc provides shared ownership)
   - **DECISION**: Keep state in `server/handlers.rs` for now, it's fine

3. **REST API Handlers** (`src/server/handlers.rs`) - **EXISTS, SKIP FOR NOW**
   - Already implemented: `/api/stellarhosts`, `/api/exoplanets`, schemas
   - Don't touch for Task 2
   - Future: add `/api/stats` endpoint using same functions as Leptos

4. **Server Functions** - **TO BE IMPLEMENTED**
   - Create `get_overview_stats()` server function (probably in `components/overview.rs`)
   - Access dataframes from shared state
   - Call aggregation functions from `tables` module
   - Return stats to frontend component

#### 2.3.2. Frontend Components
1. **Overview Page Component** (`src/components/overview.rs`)
   - Main entry point at `/` and `/overview` routes
   - Calls `get_overview_stats()` server function
   - Displays statistics in responsive grid layout (use tailwindcss)
   - Loading states and error handling

2. **Statistics Display Components**
   - Stat cards for key metrics (counts, averages)
   - Breakdown sections for distributions (discovery methods, planet sizes)
   - Responsive design using Tailwind CSS

### 2.4. Data Aggregations to Display

#### 2.4.1. Summary Statistics (Top Cards) - **VERIFIED COLUMNS EXIST**
Based on `src/tables/commands.rs`, these columns exist:
- **Stellar Hosts Count**: `stellarhosts_df.height()` ✅
- **Exoplanets Count**: `exoplanets_df.height()` ✅
- **Average Stellar Temperature**: Mean of `st_teff` column ✅ (see line 200 in commands.rs)
- **Average System Distance**: Mean of `sy_dist` column ✅ (see line 209 in commands.rs)

#### 2.4.2. Discovery Methods Distribution - **VERIFIED COLUMN EXISTS**
- Column `discoverymethod` exists in exoplanets table ✅ (see line 254 in commands.rs)
- Breakdown of exoplanets by discovery method (e.g., Transit, Radial Velocity)
- Display top 10 methods with planet counts
- Sorted by frequency (descending)

#### 2.4.3. Planet Size Categories - **OPTIONAL FOR NOW**
- Column `pl_rade` (planet radius in Earth radii) exists ✅ (see line 260 in commands.rs)
- Classification based on radius:
  - Sub-Earth: < 1.0 R⊕
  - Earth-like: 1.0 - 1.5 R⊕
  - Super-Earth: 1.5 - 2.5 R⊕
  - Neptune-like: 2.5 - 4.0 R⊕
  - Jupiter-like: > 4.0 R⊕
- **NOTE**: Already partially implemented in `src/components/overview.rs:242-258`

### 2.5. Technical Implementation Details

#### 2.5.1. Tables Module API (Internal) - **ALREADY SEPARATED**
Current structure in `src/tables/`:
```
src/tables/
├── mod.rs              - Module declarations
├── common.rs           - Shared utilities (load_parquet, get_numeric_stats, etc.)
├── stellarhosts.rs     - Stellarhosts-specific functions (load_data)
├── exoplanets.rs       - Exoplanets-specific functions (load_data)
├── aggregation.rs      - Shared aggregation functions
├── commands.rs         - CLI command implementations
├── votable_loader.rs   - VOTable parsing
└── conversion.rs       - VOT to Parquet conversion
```

**DECISION**: Each dataset already has its own file. Good separation! ✅ 

#### 2.5.2. REST API Endpoints - **EXISTS, DEFER STATS ENDPOINT**
Current endpoints (already implemented in `src/server/handlers.rs`):
```
GET /api/stellarhosts          # ✅ Already exists (line 75)
GET /api/exoplanets            # ✅ Already exists (line 76)
GET /api/stellarhosts/schema   # ✅ Already exists (line 77)
GET /api/exoplanets/schema     # ✅ Already exists (line 78)
```

**DEFER TO LATER**:
```
GET /api/stats                 # TODO: Add later using same logic as Leptos server function
```

For Task 2, we'll skip adding `/api/stats` endpoint. It can be added later to share the same aggregation logic.

#### 2.5.3. Server Function Signature
Reference implementation already exists in `src/components/overview.rs:166`:
```rust
#[server(GetStats, "/api/stats")]
pub async fn get_stats() -> Result<DataStats, ServerFnError>
```

**PROBLEM**: Current implementation makes HTTP requests to localhost (line 175-186) - inefficient!
**TODO**: Refactor to access dataframes directly from shared state

See Leptos example: https://github.com/leptos-rs/leptos/tree/main/examples/counter_isomorphic

#### 2.5.4. State Initialization Flow
1. Server startup (`main.rs`)
2. Load catalog from parquet files
3. Wrap in `AppState` with `Arc` for sharing
4. Provide to Axum via `.with_state()`
5. Provide to Leptos via `provide_context()`

### 2.6. Non-Goals (Future Tasks)

The following features are **explicitly excluded** from Task 2:
- ❌ Detailed data tables with sortable columns
- ❌ Search functionality
- ❌ Individual system/planet detail pages
- ❌ Data visualization charts/graphs
- ❌ User preferences or settings
- ❌ Data export functionality
- ❌ OpenAPI/Swagger documentation (REST API works, docs come later)

### 2.7. Success Criteria

#### 2.7.1. Functional Requirements
- ✅ Overview page loads at `/` route
- ✅ Displays all 4 summary statistics correctly
- ✅ Shows discovery methods distribution (top 10)
- ✅ Shows planet size categories distribution
- ✅ Loading state visible during data fetch
- ✅ Error state displays if data fetch fails
- ✅ Page is responsive (mobile and desktop)

#### 2.7.3. Architecture Requirements
- ✅ REST API endpoints functional and queryable
- ✅ Server functions access data directly (no HTTP self-calls)
- ✅ Single data load at startup (no redundant loading)
- ✅ AppState shared between Axum and Leptos

#### 2.7.4. Code Quality Requirements
- ✅ Clear separation: tables / server / frontend
- ✅ No business logic in HTTP handlers
- ✅ Type-safe data structures throughout
- ✅ Proper error handling with meaningful messages

### 2.8. Current File Structure - **UPDATED**

```
src/
├── tables/               [EXISTS] ✅
│   ├── mod.rs           - Module declarations
│   ├── common.rs        - Shared utilities (load_parquet, get_numeric_stats, etc.)
│   ├── stellarhosts.rs  - Stellarhosts data loading
│   ├── exoplanets.rs    - Exoplanets data loading
│   ├── aggregation.rs   - [EXTEND] Add simple overview aggregations
│   ├── commands.rs      - CLI commands
│   ├── votable_loader.rs - VOTable parsing
│   └── conversion.rs    - VOT to Parquet conversion
├── server/              [EXISTS] ✅
│   ├── mod.rs           - Module declarations
│   ├── handlers.rs      - [KEEP] REST API endpoints + ApiState
│   └── tests.rs         - Tests
├── components/          [EXISTS] ✅
│   ├── mod.rs           - Module declarations
│   └── overview.rs      - [REFACTOR] Fix server function to use direct state access
├── app.rs               [EXISTS] ✅ - Routes already configured
├── main.rs              [EXISTS] ✅ - State already initialized in handlers.rs
└── lib.rs               [EXISTS] ✅ - Module declarations
```

**What needs to change**:
1. `src/tables/aggregation.rs` - Add 4 simple aggregation functions for overview
2. `src/components/overview.rs` - Fix server function to access state directly (not via HTTP)
3. Frontend components already exist, may need minor tweaks

### 2.9. Implementation TODOs

#### Phase 1: Understand Current State (Research)
- [ ] 1. Read Leptos server function docs to understand how to access shared state
- [ ] 2. Look at Leptos counter_isomorphic example for state access pattern
- [ ] 3. Check how ApiState is currently created and shared in handlers.rs
- [ ] 4. Understand how to share state between Axum and Leptos (via `provide_context`)
- [ ] 5. Document findings on state sharing approach

#### Phase 2: Create State Sharing Architecture
- [ ] 6. Decide on state structure: keep ApiState or create new AppState?
- [ ] 7. Move state initialization from handlers.rs to main.rs (if needed)
- [ ] 8. Create state wrapper that can be shared with both Axum and Leptos
- [ ] 9. Add state to Leptos context in main.rs
- [ ] 10. Test that state is accessible from both Axum handlers and Leptos

#### Phase 3: Add Aggregation Functions to tables/aggregation.rs
- [ ] 11. Add function `calculate_total_counts(stellarhosts_df, exoplanets_df) -> (usize, usize)`
- [ ] 12. Add function `calculate_avg_temperature(df: &DataFrame) -> Option<f64>`
- [ ] 13. Add function `calculate_avg_distance(df: &DataFrame) -> Option<f64>`
- [ ] 14. Add function `calculate_discovery_methods(df: &DataFrame) -> Vec<(String, usize)>`
- [ ] 15. Test each function individually with small dataset
- [ ] 16. Add unit tests for aggregation functions

#### Phase 4: Refactor Server Function in components/overview.rs
- [ ] 17. Read current implementation in overview.rs (lines 166-197)
- [ ] 18. Remove HTTP request code (lines 175-186)
- [ ] 19. Add code to extract state from Leptos context
- [ ] 20. Call aggregation functions from tables module with dataframes
- [ ] 21. Update DataStats struct if needed to match new data
- [ ] 22. Test server function compiles without errors

#### Phase 5: Update Frontend Components (if needed)
- [ ] 23. Review StatsOverview component (line 68)
- [ ] 24. Fix temperature display formatting (currently divides by 1000, line 85)
- [ ] 25. Review StatCard components for correct styling
- [ ] 26. Review DetailedStats section (line 100)
- [ ] 27. Test responsive layout on mobile and desktop

#### Phase 6: Integration and Testing
- [ ] 28. Build the project: `cargo build`
- [ ] 29. Fix any compilation errors
- [ ] 30. Start server: `cargo leptos serve`
- [ ] 31. Open browser and navigate to http://localhost:3000
- [ ] 32. Verify overview page loads without errors
- [ ] 33. Check browser console for JavaScript errors
- [ ] 34. Verify all 4 stat cards display correct data
- [ ] 35. Verify discovery methods section displays correctly
- [ ] 36. Test SSR: disable JavaScript and verify page still renders
- [ ] 37. Test hydration: enable JavaScript and verify interactivity

#### Phase 7: Polish and Documentation
- [ ] 38. Add code comments to new aggregation functions
- [ ] 39. Update README if needed with new features
- [ ] 40. Verify error handling shows proper messages
- [ ] 41. Check loading state displays correctly
- [ ] 42. Test with missing data (empty parquet files)
- [ ] 43. Final review of all changes

### 2.10. Testing Approach

- **Catalog Module**: Unit tests for aggregation calculations
- **REST API**: Manual testing with curl/Postman
- **Server Functions**: Browser testing with SSR enabled
- **Frontend**: Visual testing in browser (desktop + mobile)
- **Performance**: Measure load times with browser DevTools
