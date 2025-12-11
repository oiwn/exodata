# Current Context: Stellar Hosts Data Table - Status & Next Steps

# IDEAS

- [ ] each page should have address, for table this mean it will include parameters, i would like to have api endpoints if i just add ".json" at the end of address.
- [ ] when sorting table by some parameter need to exclude empty (no data).

## Completed Work ✅

### Implementation Complete
We successfully implemented the basic stellar hosts table feature:

1. **Server Function** (`get_stellarhosts_page`) ✅
   - Takes page, limit, sort_by, order parameters
   - Selects 5 columns: hostname, sy_dist, st_teff, st_mass, sy_pnum
   - Applies sorting and pagination using Polars
   - Returns JSON via TableData struct
   - **Working**: Server logs confirm function executes successfully, returns 50 rows

2. **Table Component** (`table.rs`) ✅
   - Reusable presentation component
   - Renders headers with sort indicators
   - Formats cells (nulls as "—", numbers with 2 decimals)
   - Clickable column headers for sorting

3. **Page Component** (`stellarhosts_table.rs`) ✅
   - Manages reactive state (page, sort_column, sort_order)
   - Resource that fetches data on state changes
   - Pagination controls (Previous/Next buttons)
   - Loading and error states

4. **Integration** ✅
   - Routes configured: `/stellarhosts`
   - Navigation link from overview page
   - Components exported in mod.rs

5. **Configuration Fixes** ✅
   - Moved Axum REST API to `/rest/*` (was conflicting with Leptos `/api/*`)
   - Fixed `#[server]` macro syntax for Leptos 0.8
   - Fixed client-side imports and stub functions

## Known Issues ⚠️

### Critical Bugs Identified

The implementation has significant stability issues:

1. **Initial Load Fails**
   - Table does not load on first click to `/stellarhosts`
   - Requires page refresh (F5) to display data
   - **Evidence**: `Errors: [ NotFound, ]` in server logs despite successful data fetch

2. **Sorting Hangs**
   - Clicking column headers shows infinite "Loading..." indicator
   - Server function executes successfully (confirmed in logs)
   - Client never receives response or fails to process it

3. **Pagination Not Working**
   - Previous/Next buttons likely have same issue as sorting
   - Untested due to initial load failures

4. **Navigation Issues**
   - Back button from table to overview shows errors
   - State not properly maintained between routes
   - Requires full page reload to fix

5. **Suspected Root Cause**
   - `NotFound` error suggests asset loading issue (WASM, CSS) OR routing conflict
   - Server function works but client-side hydration/rendering failing
   - Possible serialization/deserialization mismatch between server and client

### What's Actually Working

From server logs we know:
- ✅ Server function registration works
- ✅ Data loading from Parquet works (46,887 rows)
- ✅ Polars operations work (select, sort, pagination)
- ✅ JSON serialization works (50 rows returned)
- ❌ Client-side rendering/hydration broken

### Root Cause Analysis

**Problem Identified**: Client-side WASM panic - `unreachable!()` at line 17

```
panicked at src/components/stellarhosts_table.rs:17:5:
internal error: entered unreachable code
```

**Why it happened:**
1. Manual client-side stub function hit `unreachable!()` instead of calling server
2. Server module was `#[cfg(feature = "ssr")]` only - client couldn't see server functions
3. Helper functions without proper `#[cfg]` guards compiled on client (WASM)
4. Polars/server deps tried to compile for WASM → compilation errors

**Lesson learned**: Conditional compilation (`#[cfg(feature = "ssr")]`) is tricky and error-prone when mixing server-only code with shared types.

## Architectural Decision: Common Business Logic Layer

### The Problem

Both Leptos server functions and Axum REST handlers need to do the **same operations**:
1. Take parameters (page, limit, sort_by, order)
2. Access DataFrame from ApiState
3. Apply Polars operations (select, sort, paginate)
4. Convert to JSON
5. Return results

Currently: **Code duplication** and **confusion** about where logic belongs.

### The Solution: Three-Layer Architecture

```
┌─────────────────────────────────────┐
│  Transport Layer (HTTP/RPC)         │
├─────────────────────────────────────┤
│  functions.rs  │  handlers.rs       │  ← Thin wrappers
│  (Leptos)      │  (Axum)            │
└────────┬───────┴──────┬─────────────┘
         │              │
         └──────┬───────┘
                ↓
┌─────────────────────────────────────┐
│  Business Logic Layer               │
├─────────────────────────────────────┤
│  common.rs                          │  ← Core logic (server-only)
│  - Pure functions                   │
│  - No HTTP/Leptos deps              │
│  - Easy to test                     │
└─────────────────────────────────────┘
                ↓
┌─────────────────────────────────────┐
│  Data Layer                         │
├─────────────────────────────────────┤
│  ApiState (Arc<DataFrame>)          │
└─────────────────────────────────────┘
```

### Structure

```
src/server/
├── common.rs       [NEW] Core business logic (pure functions)
│                   - #[cfg(feature = "ssr")] on entire module
│                   - get_stellarhosts_data(df, page, limit, sort, order)
│                   - Returns (rows, total, columns) or error
│                   - No HTTP, no Leptos, just Polars + logic
│
├── functions.rs    Thin Leptos wrappers
│                   - Visible to both client and server
│                   - #[server] macro on each function
│                   - Extracts ApiState from context
│                   - Calls common::* functions
│                   - Wraps result in TableData
│
└── handlers.rs     Thin Axum wrappers
                    - #[cfg(feature = "ssr")] on entire module
                    - Extracts State and Query
                    - Calls common::* functions
                    - Wraps result in Json<ApiResponse>
```

### Benefits

✅ **Single source of truth** - Business logic in one place
✅ **Easy testing** - Test `common.rs` without HTTP/Leptos overhead
✅ **DRY** - No duplication between handlers and functions
✅ **Flexibility** - Same logic via REST API AND server functions
✅ **Clean separation** - Transport vs business logic
✅ **SSR optimization** - Server functions have zero HTTP overhead
✅ **External API** - REST endpoints available for other clients
✅ **Clear compilation** - No confusion about what compiles where

### Why This Works for SSR

**During SSR (server-side):**
```rust
// Leptos server function executes directly (no HTTP)
#[server]
async fn get_stellarhosts_page(...) -> Result<TableData, _> {
    let state = expect_context::<ApiState>();
    let (rows, total, cols) = common::get_stellarhosts_data(&state.df, ...)?;
    Ok(TableData { rows, total, ... })
}
// ↑ Zero overhead, direct function call
```

**After hydration (client-side):**
```rust
// Client calls same function, #[server] macro generates HTTP stub
get_stellarhosts_page(...).await
// ↑ Makes POST to /api/get_stellarhosts_page automatically
```

**REST API (external clients):**
```rust
// Traditional REST endpoint for external tools/testing
GET /api/stellarhosts?page=1&limit=50
// ↑ Calls same common::get_stellarhosts_data() function
```

### Implementation Plan

1. **Create `src/server/common.rs`**
   - Pure business logic functions
   - All marked `#[cfg(feature = "ssr")]`
   - No HTTP/Leptos dependencies
   - Returns simple Result types

2. **Refactor `functions.rs`**
   - Keep `#[server]` functions
   - Remove all Polars logic
   - Call `common::*` functions
   - Handle context extraction and error mapping

3. **Update `handlers.rs`**
   - Keep Axum handlers
   - Remove duplicate logic
   - Call same `common::*` functions
   - Handle HTTP-specific concerns

4. **Add unit tests**
   - Test `common.rs` functions directly
   - No HTTP server needed
   - Fast, reliable tests

### Decision Rationale

**Why not pure REST API?**
- Server functions optimize SSR (no HTTP overhead)
- Type-safe client/server communication
- Integrated with Leptos patterns

**Why not only server functions?**
- REST API useful for testing (curl, Postman)
- External integrations may need it
- Standard HTTP patterns

**Why both?**
- Best of both worlds
- Minimal code duplication (shared `common.rs`)
- Flexible for future needs

## Next Immediate Steps

### Priority 1: Implement Clean Architecture ⏭️

Follow the implementation plan above to create the three-layer architecture.

**Steps:**
1. Create `src/server/common.rs` with core business logic
2. Refactor `functions.rs` to thin wrappers
3. Update `handlers.rs` to use common logic
4. Verify compilation (both SSR and WASM)
5. Test in browser

**Success criteria:**
- Code compiles cleanly
- Table loads without errors
- Sorting works
- Pagination works
- No WASM panics

### Priority 2: Add Testing Infrastructure 🧪

Once the architecture is stable:

1. **Unit tests for `common.rs`**
   - Test Polars operations in isolation
   - Test pagination edge cases
   - Test sorting logic
   - Test error handling

2. **Integration tests**
   - Test Axum handlers with test server
   - Verify JSON response format
   - Test all query parameters

3. **E2E tests** (future)
   - Browser automation with Playwright
   - User interaction flows
   - Visual regression testing

### Priority 3: Documentation 📝

- Document the three-layer architecture
- Add code comments explaining `#[cfg]` usage
- Create developer guide for adding new endpoints
- Document testing approach

---

## Current Status

**Blocked**: Code doesn't compile due to improper conditional compilation setup.

**Next action**: Implement the common.rs architecture to unblock development.

