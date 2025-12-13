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

