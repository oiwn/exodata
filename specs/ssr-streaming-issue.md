# SSR Streaming Failure on Table Routes

## Problem

Browser shows `net::ERR_INCOMPLETE_CHUNKED_ENCODING 200 (OK)` when loading `/exoplanets` or `/stellarhosts` in production. Response stream starts with 200 and chunked transfer, sends partial body (~49KB), then stalls until timeout.

## Environment

- DigitalOcean droplet: s-1vcpu-2gb (1 vCPU, 2GB RAM)
- Docker, Nginx reverse proxy -> 127.0.0.1:3000
- Axum + Leptos 0.8 SSR with streaming

## Key Observations

| Symptom | Observation |
|---------|-------------|
| Direct curl to app also stalls | Issue is in app, not Nginx |
| REST API returns instantly | `/rest/exoplanets?page=1&limit=50` returns in ~1.7ms |
| ~49KB partial response | Shell renders, then stalls waiting for Resource |
| Cache prewarm works | Runs at startup with no SSR contention |
| Works in dev | More resources, different timing |

## Root Cause

CPU-bound Polars operations in `get_table_data()` (src/server/common.rs:33-146) run synchronously on the async executor during SSR streaming. On a single-vCPU machine, this blocks the tokio runtime during the critical SSR rendering phase.

**Why REST works but SSR hangs:**
- REST: Simple request -> await result -> return JSON
- SSR: Complex streaming pipeline where Leptos coordinates rendering while resolving `Resource` futures. When Polars hogs the executor, the streaming stalls.

## Solutions

### Solution 1: Wrap Polars in `spawn_blocking` (Recommended)

In `src/server/common.rs`, wrap the calls inside `get_*_data_cached()` functions:

```rust
let df = df.clone();
let all_metadata = all_metadata.clone();
let sort_by_clone = sort_by.clone();
let order_clone = order.clone();
let selected_columns_clone = selected_columns.clone();
let filter_clone = filter.clone();

let result = tokio::task::spawn_blocking(move || {
    get_stellarhosts_data(
        &df,
        &all_metadata,
        page,
        limit,
        sort_by_clone,
        order_clone,
        selected_columns_clone,
        filter_clone,
    )
})
.await
.map_err(|e| format!("spawn_blocking error: {}", e))?;
```

This offloads CPU work to the blocking thread pool, keeping the async executor free for SSR streaming.

### Solution 2: Increase blocking thread pool

In `src/main.rs`, configure tokio with more threads:

```rust
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
```

Or use `tokio::runtime::Builder` to increase `max_blocking_threads`.

### Solution 3: Upgrade droplet

The s-1vcpu-2gb is marginal for SSR + Polars. A 2-vCPU instance would give the runtime more breathing room.

## Files Affected

- `src/server/common.rs` - cached data functions
- `src/server/functions.rs` - Leptos server functions (callers)
- `src/server/handlers.rs` - REST handlers (callers, already use spawn_blocking for SQL)

## Status

- [ ] Implement Solution 1 (spawn_blocking)
- [ ] Test locally with production build
- [ ] Deploy and verify on production
