# Current Context: Routing & Navigation Issues

## Session Summary: What We Accomplished

### ✅ Completed: Three-Layer Architecture Implementation

We successfully implemented the clean architecture from the previous plan:

1. **Created `src/server/common.rs`** - Pure business logic for data operations
   - `get_stellarhosts_data()` function with Polars operations
   - Isolated from HTTP/Leptos concerns
   - Unit tests included
   - Easy to test and maintain

2. **Refactored `src/server/functions.rs`** - Thin Leptos server function wrappers
   - `get_stellarhosts_page()` now just extracts context and calls `common::`
   - `get_stats()` does the same for overview stats
   - Fixed `serde_json::Value` import to use `leptos::serde_json::Value` for WASM compatibility

3. **Updated `src/server/mod.rs`** - Exported common module properly

4. **Build Success** ✅
   - Server-side compilation works (`cargo build --features ssr`)
   - WASM compilation works (`cargo leptos build`)
   - Three-layer architecture compiles cleanly

### ✅ Partial Success: Stellar Hosts Table

The stellar hosts table at `/stellarhosts` is **working**:
- Loads data successfully
- Displays 50 rows with 5 columns
- Pagination controls present
- Sorting UI present

## 🚨 Critical Issues Discovered

### Issue 1: Server Functions Not Registered for HTTP Calls

**Problem**: Server functions work during SSR (in-process calls) but fail when called via HTTP POST from the client.

**Symptoms**:
- Initial page load at `/overview` works ✅ (SSR renders with data)
- Navigate to `/stellarhosts` works ✅
- Navigate back to `/overview` FAILS ❌ with:
  ```
  POST http://127.0.0.1:3000/api/get_stats11877934666105900369
  Status: 404 Not Found
  Error: "Could not find a server function at the route /api/get_stats..."
  ```

  ^^^ why it's POST not get? We should not transmit any data into the API. Is it possible to make all api requests GET (server functions) ? Replace this comment with remark, you need to find out the answer!

**Root Cause**:
`leptos_routes_with_context()` does **NOT** automatically set up `/api/*` routes for server function HTTP endpoints. It only sets up:
- SSR page rendering routes
- Context provision during SSR

^^^ No idea is it's true. "https://raw.githubusercontent.com/leptos-rs/leptos/refs/heads/main/examples/server_fns_axum/src/app.rs" i think here is example of everything. 

Server functions need **explicit HTTP route handlers**.

**Evidence**:
- Server logs show: `Errors: [ NotFound, ]`
- Client receives HTML 404 page instead of JSON
- Error message: "missing delimiter" (trying to parse HTML as JSON)

### Issue 2: Confusing Duplicate Code in `overview.rs`

Lines 10-27 in `src/components/overview.rs`:

```rust
// DataStats struct for client-side (must match server definition)
#[cfg(not(feature = "ssr"))]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DataStats {
    // ... duplicate definition ...
}

#[cfg(not(feature = "ssr"))]
#[leptos::server]
pub async fn get_stats() -> Result<DataStats, leptos::server_fn::ServerFnError> {
    // This will be replaced by actual implementation on the server
    unreachable!()
}
```

**Problems**:
- Duplicate `DataStats` definition (also in `src/server/functions.rs`)
- Client-side stub with `unreachable!()` - unclear if needed
- Confusing conditional compilation
- Might be causing server function registration issues

### Issue 3: Poor UX - No Navigation Menu

**Current State**:
- Root page `/` shows overview stats
- Navigation to `/stellarhosts` via button at bottom of stats
- No way to navigate back except browser back button
- No persistent navigation menu

**Problems**:
- Button at bottom is terrible UX
- No clear navigation structure
- Can't directly access pages via URL parameters
- Doesn't align with ideas.md goals

### Issue 4: No URL-Based State (ideas.md Requirements)

From `specs/ideas.md`:
> - [ ] each page should have address, for table this mean it will include parameters,
> - [ ] would be cool to route to api endpoints if i just add ".json" at the end of address.

**Current State**:
- Table state (page, sort, filters) lives only in client-side signals
- No URL query parameters
- Can't bookmark or share a specific table view
- No `.json` endpoint variant

**What We Need**:
- `/stellarhosts?page=2&sort=sy_dist&order=desc` - URL reflects table state
- `/stellarhosts.json?page=2` - JSON API variant
- URL updates as user interacts with table
- Direct navigation to any table state via URL

## 🔍 Open Questions

### Q1: Server Function Registration Strategy

**Options**:

**A) Explicit Route Handler (Previous Attempt)**
```rust
.route("/api/{*fn_name}", axum::routing::post(
    handle_server_fns_with_context(provide_api_state)
))
```
- We tried this, removed it thinking leptos_routes_with_context handled it
- Actually, we DO need this!

**B) Use Different Prefix**
```rust
#[server(prefix = "/server-fns")]
pub async fn get_stats() -> Result<...>
```
- Avoid potential conflicts with leptos routing
- Requires explicit handler at `/server-fns/*`

**C) Explicit Registration in main()**
```rust
GetStats::register_explicit()?;
GetStellarhostsPage::register_explicit()?;
```
- Error message suggested this
- Manual but explicit

**Which approach is correct?**

### Q2: Client-Side Stubs Needed?

Do we need the `#[cfg(not(feature = "ssr"))]` client-side definitions and stubs?
- The `#[server]` macro supposedly generates client stubs automatically
- But maybe we need matching type definitions on both sides?
- Why is there an `unreachable!()` stub?

### Q3: Route Structure

**Current**:
```
/              -> OverviewPage
/overview      -> OverviewPage (duplicate)
/stellarhosts  -> StellarHostsTablePage
```

**Should we**:
- Remove root `/` route, use `/overview` as canonical?
- Add redirect from `/` to `/overview`?
- Keep both but fix server function routing first?

## 📋 Next Steps Plan

### Priority 1: Fix Server Function HTTP Routes ⚠️

**Goal**: Make server functions accessible via POST `/api/*` for client calls.

**Approach**:
1. Add explicit server function handler to `main.rs` **BEFORE** `leptos_routes_with_context`
2. Use the correct Axum 0.8 wildcard syntax: `/api/{*fn_name}`
3. Ensure `handle_server_fns_with_context` receives the same context as SSR
4. Test both `get_stats` and `get_stellarhosts_page` work from client

**Success Criteria**:
- Navigate from `/stellarhosts` back to `/overview` without errors
- Browser Network tab shows successful POST to `/api/get_stats...`
- No 404 errors in server logs

### Priority 2: Clean Up Duplicate Definitions 🧹

**Goal**: Remove confusing client-side stubs and duplicate types.

**Actions**:
1. Remove lines 10-27 from `src/components/overview.rs`
2. Import `DataStats` and `get_stats` from `crate::server::functions` on both client and server
3. Verify the `#[server]` macro generates proper client stubs without our manual stubs
4. Test SSR and client hydration still work

### Priority 3: Implement Proper Navigation 🧭

**Goal**: Add persistent navigation menu, clean up routing.

**Actions**:
1. Create `src/components/nav.rs` - Navigation component
   - Links to: Overview, Stellar Hosts, (future: Exoplanets)
   - Sticky header with current page highlighting
   - Clean, space-themed design
2. Remove navigation button from bottom of overview stats
3. Add nav component to shell/layout
4. Consider: Remove root `/` route or add redirect to `/overview`

### Priority 4: URL-Based Table State 🔗

**Goal**: Table state in URL query parameters (ideas.md requirement).

**Approach**:
1. Use Leptos router's query parameter support
2. Read page/sort/order from URL on mount
3. Update URL when user changes page/sorting
4. Make table state bookmarkable and shareable

**Implementation**:
- Use `use_query_map()` from leptos_router
- Parse `?page=2&sort=sy_dist&order=desc`
- Update signals from URL params
- Use `use_navigate()` to update URL when state changes

### Priority 5: JSON API Endpoints (Optional) 📊

From ideas.md:
> would be cool to route to api endpoints if i just add ".json" at the end of address.

**Approach**:
- Add route handlers for `/{page}.json` variants
- Return same data as REST API (`/rest/*`) in JSON format
- Example: `/stellarhosts.json?page=2` returns raw table data

## 🏗️ Architecture Status

### What's Working ✅
- **Three-layer architecture**: Clean separation of concerns
- **SSR rendering**: Pages render with data on initial load
- **Data layer**: Polars operations in `common.rs`
- **Thin wrappers**: Server functions and handlers call common logic
- **Build system**: Both server and WASM compile cleanly

### What's Broken ❌
- **Server function HTTP routing**: 404 on client-initiated calls
- **Client-side type definitions**: Duplicate and confusing
- **Navigation**: No persistent menu, poor UX
- **URL state**: Table state not in URL

### Next Session Focus 🎯

**Start with Priority 1** - Fix server function routing. Everything else depends on this working correctly.

Once server functions work reliably, tackle the UX improvements (navigation, URL state) to align with project goals in ideas.md.

## Technical Notes

### Server Function Registration Research

From documentation and testing:
- `#[server]` macro generates unique URL paths (function name + hash)
- Registration happens at compile time
- `leptos_routes_with_context` handles SSR + context provision
- `handle_server_fns_with_context` needed for HTTP endpoint handling
- Both must receive the same context closure

### Current main.rs Setup

```rust
.leptos_routes_with_context(
    &leptos_options,
    routes,
    provide_api_state.clone(),  // Context for SSR
    move || shell(leptos_options.clone())
)
.nest_service("/rest", server::api_routes(api_state))
.fallback(leptos_axum::file_and_error_handler(shell))
```

**Missing**: Explicit `/api/*` handler for server function HTTP calls!

### Files Modified This Session

- ✅ `src/server/common.rs` - Created with business logic
- ✅ `src/server/functions.rs` - Refactored to thin wrappers
- ✅ `src/server/mod.rs` - Added common module export
- ✅ `src/main.rs` - Updated routing (attempted server function handler)
- ✅ `src/components/table.rs` - Fixed `serde_json::Value` import
- ⏭️ `src/components/overview.rs` - Needs cleanup (duplicate definitions)
- ⏭️ `src/app.rs` - Needs navigation menu addition
- ⏭️ `src/components/stellarhosts_table.rs` - Needs URL param integration

## References

- specs/ideas.md - Project goals (URL state, .json endpoints)
- specs/architecture.md - Workspace structure
- specs/web-frontend.md - Leptos UI patterns
- specs/web-backend.md - Server function patterns

---

**Session Status**: Architecture complete, server function routing broken, UX needs improvement.

**Ready for**: Priority 1 implementation - Fix server function HTTP routes.
