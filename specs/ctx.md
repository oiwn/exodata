# Exoplanet Catalog - Task 2: Overview Page Implementation

## Current Status
- Frontend components already exist in `src/components/overview.rs`
- ApiState exists in `src/server/handlers.rs` with data loading
- REST API endpoints work (`/api/stellarhosts`, `/api/exoplanets`)
- Server function makes inefficient HTTP requests (needs refactoring)
- Routes configured in `src/app.rs` for "/" and "/overview"

## Implementation Checklist

### Phase 1: Add Simple Aggregation Functions (tables/aggregation.rs)
- [ ] Add function `get_total_counts(stellarhosts_df: &DataFrame, exoplanets_df: &DataFrame) -> (usize, usize)` that returns tuple of heights
- [ ] Add function `get_avg_temperature(df: &DataFrame) -> Option<f64>` that calculates mean of `st_teff` column
- [ ] Add function `get_avg_distance(df: &DataFrame) -> Option<f64>` that calculates mean of `sy_dist` column
- [ ] Add function `get_discovery_methods(df: &DataFrame, limit: usize) -> Vec<(String, usize)>` that groups by `discoverymethod` and returns top N
- [ ] Add function `get_planet_size_categories(df: &DataFrame) -> Vec<(String, usize)>` that categorizes by `pl_rade` radius
- [ ] Compile and verify no errors: `cargo check`

### Phase 2: Create Shared State Structure (new file or modify handlers.rs)
- [ ] In handlers.rs, make ApiState derive Clone if needed (already has Clone)
- [ ] Verify ApiState has pub fields: `pub stellarhosts_df: Arc<DataFrame>` and `pub exoplanets_df: Arc<DataFrame>`
- [ ] Compile and verify: `cargo check`

### Phase 3: Refactor main.rs to Share State with Leptos
- [ ] In main.rs `start_server()`, load stellarhosts dataframe before creating Router (around line 135)
- [ ] In main.rs `start_server()`, load exoplanets dataframe before creating Router
- [ ] Create ApiState instance in main.rs from the loaded dataframes
- [ ] Pass ApiState to `server::api_routes()` function (modify api_routes signature to accept state)
- [ ] Compile and fix errors: `cargo check`

### Phase 4: Modify api_routes() to Accept State Parameter
- [ ] In handlers.rs, change `api_routes()` signature from `pub fn api_routes() -> Router` to `pub fn api_routes(state: ApiState) -> Router`
- [ ] Remove data loading code from inside `api_routes()` (lines 59-72)
- [ ] Use the passed `state` parameter in `.with_state(state)`
- [ ] Compile and verify: `cargo check`

### Phase 5: Add Leptos Context for State Sharing
- [ ] In main.rs, add code to provide ApiState to Leptos context in the shell closure (line 138)
- [ ] Use leptos `provide_context` to make ApiState available to server functions
- [ ] Compile and verify: `cargo check`

### Phase 6: Implement Server Function in server/functions.rs
- [ ] In server/functions.rs, inside `get_stats()`, add code to extract ApiState from leptos context using `use_context::<ApiState>()`
- [ ] Add error handling if context is missing: return ServerFnError with message
- [ ] Compile and fix syntax errors: `cargo check`

### Phase 7: Use Aggregation Functions in Server Function
- [ ] Import aggregation functions: `use crate::tables::aggregation::*;`
- [ ] Call `get_total_counts()` with both dataframes from state
- [ ] Call `get_avg_temperature()` with stellarhosts dataframe
- [ ] Call `get_avg_distance()` with stellarhosts dataframe
- [ ] Call `get_discovery_methods()` with exoplanets dataframe and limit of 10
- [ ] Call `get_planet_size_categories()` with exoplanets dataframe
- [ ] Build DataStats struct from aggregation results and return Ok()
- [ ] Compile and verify: `cargo check`

### Phase 8: Update OverviewPage Component to Use New Server Function
- [ ] In components/overview.rs, remove old DataStats struct (lines 6-14)
- [ ] Import DataStats from server module: `use crate::server::functions::{DataStats, get_stats};`
- [ ] Remove old `get_stats()` server function (lines 166-197)
- [ ] Remove old `calculate_stats_server()` function (lines 199-274)
- [ ] Fix temperature display bug in line 85: remove division by 1000
- [ ] Compile: `cargo check`

### Phase 9: Build and Test
- [ ] Run full build: `cargo leptos build`
- [ ] Fix any compilation errors that appear
- [ ] Start server: `cargo leptos serve`
- [ ] Open browser to http://localhost:3000
- [ ] Verify overview page loads without errors
- [ ] Check browser console for JavaScript errors
- [ ] Verify all 4 stat cards show reasonable values
- [ ] Verify discovery methods section displays data
- [ ] Verify planet size categories section displays data

### Phase 10: Final Verification
- [ ] Test with browser DevTools Network tab to ensure no localhost HTTP calls
- [ ] Verify page loads quickly (no self-HTTP-requests)
- [ ] Test responsive design on mobile viewport
- [ ] Verify loading spinner appears briefly during SSR
- [ ] Check that error handling works (temporarily break data loading)
- [ ] Restore working code and final test

## Key Architecture Notes
- **State Flow**: main.rs loads data → creates ApiState → passes to both Axum (.with_state) and Leptos (provide_context)
- **Server Functions**: Access ApiState via `use_context()` instead of HTTP requests
- **Data Layer**: All aggregation logic lives in tables/aggregation.rs module
- **Frontend**: Already implemented, should work without changes after backend refactoring

## Files to Modify
1. `src/tables/aggregation.rs` - Add 5 new simple aggregation functions
2. `src/server/handlers.rs` - Modify api_routes() signature to accept state
3. `src/main.rs` - Move data loading, create and share ApiState
4. `src/server/functions.rs` - Implement get_stats() server function (NEW FILE)
5. `src/server/mod.rs` - Add functions module declaration (DONE)
6. `src/components/overview.rs` - Remove old server function, import from server/functions
