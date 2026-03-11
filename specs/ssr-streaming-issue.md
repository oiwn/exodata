# SSR Streaming Failure on Table Routes

## Problem

Browser shows 504 Gateway Time-out on `/exoplanets` or `/stellarhosts` in production.
Previously was `net::ERR_INCOMPLETE_CHUNKED_ENCODING` (stalling at ~49KB), now escalated to
full timeout.

## Environment

- DigitalOcean droplet: s-1vcpu-2gb (1 vCPU, 2GB RAM)
- Docker, Nginx reverse proxy → 127.0.0.1:3000
- Axum + Leptos 0.8 SSR

## Root Cause (Revised)

**`SsrMode::Async` holds the HTTP connection open until all resources resolve.**

In `src/app.rs:91,97`:
```rust
<Route path=StaticSegment("stellarhosts") view=StellarHostsTablePage ssr=SsrMode::Async />
<Route path=StaticSegment("exoplanets")   view=ExoplanetsTablePage   ssr=SsrMode::Async />
```

`SsrMode::Async` tells Leptos: wait for ALL resources to resolve before sending any HTML.
On a 1-vCPU machine, under any load or contention, this can easily exceed Nginx's default
`proxy_read_timeout` (60s), producing a 504.

**Why `spawn_blocking` wasn't the fix:**
`spawn_blocking` for Polars is already implemented (`server/common.rs:283-299, 345-361`).
It offloads CPU work but doesn't change the fundamental problem: `SsrMode::Async` still
holds the connection open waiting for the result before sending a single byte.

**Why REST API works fine:**
REST handlers at `/rest/exoplanets` return a simple JSON response — no SSR streaming
pipeline, no Leptos resource coordination overhead.

## Chosen Solution: Change to `SsrMode::OutOfOrder`

**File**: `src/app.rs`, lines 91 and 97.

```rust
// Before:
<Route path=StaticSegment("stellarhosts") view=StellarHostsTablePage ssr=SsrMode::Async />
<Route path=StaticSegment("exoplanets")   view=ExoplanetsTablePage   ssr=SsrMode::Async />

// After:
<Route path=StaticSegment("stellarhosts") view=StellarHostsTablePage ssr=SsrMode::OutOfOrder />
<Route path=StaticSegment("exoplanets")   view=ExoplanetsTablePage   ssr=SsrMode::OutOfOrder />
```

`SsrMode::OutOfOrder`:
- Sends the HTML shell immediately (no 504 possible)
- Streams resource data into `<Suspense>` placeholders as resources resolve
- Client patches the DOM when chunks arrive
- Requires `<Suspense>` wrappers around resource-dependent UI (check table components)

**Risk**: Low. The table components likely already have `<Suspense>` since they show loading
states. If not, adding `<Suspense fallback=...>` around the table body is straightforward.

## Alternative Solutions

### Alt 1: `SsrMode::InOrder`
Like OutOfOrder but streams in document order (simpler mental model, slightly slower). Same
benefit of immediate shell response. Good fallback if OutOfOrder causes issues.

### Alt 2: Increase Nginx `proxy_read_timeout`
```nginx
proxy_read_timeout 120s;
```
Band-aid only — doesn't fix the underlying stall, just gives more rope.

### Alt 3: Increase tokio blocking threads
In `src/main.rs`, replace `#[tokio::main]` with a custom runtime:
```rust
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .max_blocking_threads(8)
        .build()
        .unwrap()
        .block_on(start_server());
}
```
Helps if spawn_blocking is queuing up, but doesn't address the SsrMode issue.

### Alt 4: Verify cache prewarm key matches SSR request
The prewarm at startup uses `page=1, limit=50, None, None, None, None`. If the table
component's initial Resource call uses different defaults (e.g., different limit), every SSR
request is a cache miss. Add tracing to `get_stellarhosts_data_cached` and check logs.

## Files Affected

- `src/app.rs` — change `SsrMode::Async` → `SsrMode::OutOfOrder` (2 lines)
- `src/components/stellarhosts_table.rs` — verify `<Suspense>` wraps resource-dependent UI
- `src/components/exoplanets_table.rs` — same

## Status

- [x] `spawn_blocking` implemented (already done in common.rs)
- [ ] Change `SsrMode::Async` → `SsrMode::OutOfOrder`
- [ ] Verify `<Suspense>` wrappers in table components
- [ ] Test locally with `cargo leptos build --release`
- [ ] Deploy and verify on production

## History

- **Initial failure**: `ERR_INCOMPLETE_CHUNKED_ENCODING` — stream stalled at ~49KB
- **After spawn_blocking added**: Escalated to 504 Gateway Time-out (Nginx gave up waiting)
- **2026-02-18**: Still unresolved, 504 confirmed in production
- **2026-03-10**: Root cause reidentified as `SsrMode::Async` + Nginx timeout interaction
