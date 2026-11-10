# Infrastructure Direction

General direction and durable deployment decisions. The concrete implementation plan, prerequisites, and acceptance checks live in [0017-hostinger-nixos-deployment](tasks/0017-hostinger-nixos-deployment.md). Current production setup is documented in [DEPLOY.md](../DEPLOY.md) and remains in place until its replacement is verified.

## Constraints

- Target: existing Hostinger NixOS VPS, $6/month prepaid for 24 months. Inspected 2026-11-07: NixOS 26.05, x86_64, one vCPU, 3916 MiB RAM, 50 GB root disk, no swap.
- DigitalOcean retirement is handled by the maintainer after cutover; do not destroy it as part of migration.
- Prefer GitHub CI for application compilation. VPS builds are a fallback requiring a measured memory budget and swap; 4 GB alone does not establish that the release build fits.
- NixOS owns host configuration and services. Keep proxy and monitoring resident memory modest.

## Current Setup

- CI (`.github/workflows/deploy.yml`) builds `infrastructure/docker/Dockerfile` and pushes to GHCR on version change.
- Ansible (`playbooks/deploy.yml`) pulls the image and runs the container with `data_path` and `content_path` mounted read-only.
- nginx + certbot (`roles/nginx/templates/`) provide TLS, gzip (including `application/wasm`), static `/data/`, no-cache `/pkg/`, and unbuffered `/mcp`.

## 1. Nix Application Build in GitHub CI — revised 2026-11-07

- Build a Nix-native application package on a GitHub-hosted Linux runner. Pin nixpkgs and build tools; build SSR and hydration with `cargo leptos build --release --split` and locked Cargo dependencies.
- Cache build dependencies and publish the package's runtime closure, including the server, full `site/` tree, and required shared libraries. Choose the cache/artifact transport during implementation; CI needs no VPS SSH key when deployment pulls artifacts manually.
- A plain Ubuntu-built tarball is no longer the runtime contract: its ELF loader and library paths cannot be assumed to exist on NixOS.
- Build time: the current Docker build takes ~19.5 min (run 37801874769, 2026-10-08) because the single `COPY src` + `cargo leptos build` layer recompiles all dependencies on every source change. Goal: cached dependencies, so a typical release only rebuilds workspace crates. Record before/after timings.
- Optional: faster linker (mold) per issue #157.
- Package the `exodata-web` server binary and complete split WASM output. Preserve version-change and manual CI triggers; release artifacts identify the source revision.
- Removes dockerd/containerd memory, GHCR login, and buildx caching.

## 2. Declarative NixOS Host and systemd Service — revised 2026-11-07

- Store the pinned host configuration in the repository. Preserve Hostinger's disk, bootloader, cloud-init networking, and guest-agent settings.
- Use an SSH-key-authenticated sudo administrator and a separate application service identity.
- NixOS declares the application unit, package, proxy, firewall, and monitoring. Ansible is retained only for the old deployment until cutover.
- Unit: private runtime environment file where needed, `LEPTOS_OUTPUT_NAME=exodata`, absolute `LEPTOS_SITE_ROOT`, loopback listen address, `EXO_DATA_DIR`, and `EXO_CONTENT_DIR`; `Restart=on-failure`, measured `MemoryMax`, and systemd sandboxing. Data and descriptions remain mutable files outside the Nix store and read-only to the app.
- Logs: journald.
- Rollback: retain previous NixOS generations and application closures; verify switching back and forward before cutover.
- Verify HTTP health before DNS changes. Remove obsolete Docker/Ansible deployment files only after the replacement is verified.

## 3. Ferron Instead of nginx + certbot; MCP Subdomain — direction chosen

- `exodata.space`: proxy to `127.0.0.1:3000`; serve `/data/` from `data_path`; `/pkg/` no-cache; compression including `application/wasm`.
- `mcp.exodata.space`: same app process; proxy maps `/` to the app's `/mcp`.
- Open: keep or redirect `exodata.space/mcp`.
- Spike checklist (issue #43): wasm compression, unbuffered MCP streaming with long read timeout, automatic HTTPS for two hostnames, systemd operation, behavior while the app is starting (Parquet load + cache prewarm).
- Follow-up: DNS record for `mcp.`; update `docs/mcp.md`, client config examples, homepage MCP text, and sitemap/OpenAPI references.

## 4. Monitoring — decided: Grafana Cloud + Alloy

- Backend: Grafana Cloud free tier (account registered 2026-10-29). Limits checked 2026-10-29: 10k active metric series, 50 GB logs, 50 GB traces, 14-day retention, 3 users, 100k synthetic API checks per month.
- Agent: Grafana Alloy on the VPS, declared by NixOS as a systemd service.
  - Host metrics (CPU, load, memory, swap, disk usage/IO, network) via the built-in unix exporter, pushed with Prometheus remote write.
  - Logs: journald entries for the `exodata` unit shipped to Loki. Requires the systemd deploy (step 2); until then, Docker container logs.
  - Credentials (remote-write/Loki URLs, user IDs, API token) in a root-owned runtime file, not in Git or the Nix store.
- App: no code changes. Existing `tracing` + `tracing-subscriber` fmt output on stdout remains the log source.
- Dashboards: Grafana's Linux node/Node Exporter dashboards; `exodata` service memory once it runs under systemd.
- Alerts: error-log rate from Loki, host memory/disk thresholds, and a synthetic HTTP check on `exodata.space` (plus `mcp.exodata.space` after step 3). Notifications to `alerts@exodata.space` (section 6).
- Measure Alloy resident memory after rollout. If the cost is too high, fallback: node_exporter + vmagent for metrics, Fluent Bit for logs.
- Rejected:
  - Sentry: error grouping is not needed at current scale; can be added later as an in-process SDK.
  - OpenObserve: no permanent cloud free tier; self-hosting would compete for 2 GB and fail together with the host.
  - New Relic, Datadog: not pursued.

## 5. NixOS — chosen (2026-11-07)

- Supersedes the 2026-10-31 rejection: the target is now an already provisioned, prepaid NixOS host rather than an Ubuntu droplet.
- Use a pinned flake for the host and application packaging, with GitHub CI compilation preferred and NixOS generation rollback.
- Nix packaging must carry the runtime dependencies; pinning alone does not guarantee faster builds. Measure cache effectiveness and build duration.

## 6. Accounts & Contact — proposed

- Project contact via forwarding addresses on `exodata.space` (Cloudflare Email Routing if DNS is on Cloudflare, otherwise ImprovMX free tier) to the maintainer's existing inbox. No separate mailbox.
- Aliases by purpose: `alerts@` (monitoring/uptime), `acme@` (TLS certificate registration), `ops@` (third-party service signups).
- Register monitoring and other infrastructure services under these aliases.
- Configure the NixOS proxy's certificate contact as `acme@exodata.space` once forwarding is verified.
- Open: a dedicated Google account instead, if service logins must be separate from the personal Google account.

## Direction Dependencies

Host configuration → CI-built Nix application package → native service → proxy and public endpoints → monitoring. Cutover sequencing and rollout checks belong to the implementation task.

