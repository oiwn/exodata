# SSR Streaming Failure on Table Routes

## Problem

Table routes (`/exoplanets`, `/stellarhosts`) hang mid-stream in production.
Browser shows `ERR_INCOMPLETE_CHUNKED_ENCODING`. Server sends ~50KB then stalls until
client timeout. `/about` and REST API work fine.

## Environment

- DigitalOcean droplet: s-1vcpu-2gb (1 vCPU, 2GB RAM)
- Docker, Nginx reverse proxy → 127.0.0.1:3000
- Axum + Leptos 0.8 SSR with streaming

## Diagnosis

Reproduced locally with `docker run --cpus=1`. Key observations that narrowed it down:

| Test | Result |
|------|--------|
| `GET /rest/exoplanets` (direct JSON) | 1.7ms ✓ |
| `GET /about` (SSR, no Resource) | instant ✓ |
| `GET /exoplanets` (SSR, with Resource) | sends ~50KB then hangs ✗ |

The ~50KB is the HTML shell + `<Transition>` fallback (spinner). The stream stalls
exactly when Leptos tries to resolve the `Resource` and stream the out-of-order patch.

## Root Cause

**Tokio worker thread starvation on 1-vCPU.**

`#[tokio::main]` defaults `worker_threads` to the number of available CPUs — so **1 worker
thread** on a 1-vCPU machine.

Leptos SSR rendering does substantial synchronous work between await points, keeping the single worker thread continuously busy. When `spawn_blocking` (Polars data processing) completes and sends a wakeup to the `JoinHandle`, there is no free worker thread to receive it. The rendering thread is occupied. The resource future never resumes. Stream hangs.

`/about` is unaffected because it has no `Resource` — rendering completes without any `spawn_blocking` round-trip.

## Fix

Force more worker threads regardless of CPU count (`src/main.rs`):

```rust
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
```

4 threads on 1 CPU: the OS scheduler interleaves them. JoinHandle wakeups are picked up by a free thread while rendering continues on another. No deadlock.

## Other Changes Made Along the Way

- `SsrMode::Async` → `SsrMode::OutOfOrder` (`src/app.rs`) — not the root cause, but correct regardless: `Async` held the connection until all resources resolved, making timeouts worse under the original starvation condition.
- `spawn_blocking` for Polars cache misses was already implemented in `server/common.rs` before this investigation began.

## Status

- [x] Root cause identified (tokio worker thread starvation)
- [x] Fix implemented (`worker_threads = 4` in `src/main.rs`)
- [ ] Verify fix in production
