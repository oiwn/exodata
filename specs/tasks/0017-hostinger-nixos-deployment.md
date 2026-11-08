---
id: 0017-hostinger-nixos-deployment
status: draft
scope: [flake.nix, flake.lock, infrastructure/nix/**, .github/workflows/deploy.yml, Justfile, DEPLOY.md, docs/mcp.md, src/server/mcp.rs, .github/workflows/nixos.yml]
created: 2026-11-07
grade: huge
---
# Task: hostinger nixos deployment

## Plan

- [ ] Capture the installed Hostinger baseline in a pinned flake; build/evaluate `nixos-vps` in CI before adding users or services. See step 1.
^^^ what's actually there? I have clear NixOS what should i init first?
&&& NixOS is already installed and configured in `/etc/nixos/configuration.nix`. We first capture that working configuration into this repository and create `flake.nix` plus `flake.lock`; no reinstall or disk initialization is needed. Details: step 1 below.

- [ ] Add and verify the `deploy` sudo administrator before restricting root SSH; use a separate application identity. See step 2.
^^^ what for?
&&& `deploy` is your named SSH administrator for uploads and privileged system activation through sudo. The application uses a different, unprivileged service identity. This lets us restrict routine root login while retaining administrative access; passwordless sudo is a proposed choice, not implied by key-only SSH. Details: step 2 below.

- [ ] Build application and NixOS generation exclusively in GitHub CI; publish cached, signed outputs and a revision manifest. See step 3.
^^^ this should be described in details
&&& Step 3 below describes tool pinning, application and system builds, caching, published outputs, and the initial host-only CI build.

- [ ] Select cache hosting and activation tooling; add operator recipes that deploy the exact CI output without Mac/VPS compilation. See step 4.
^^^ need details
&&& Step 4 below describes cache trust, revision selection, download-only deployment, activation, and rollback. Cache hosting and deployment tooling remain decisions to review, not installed components.


- [ ] Declare the application service, upload external data/descriptions, and verify startup and rollback before cutover. See step 5.


- [ ] Deploy Ferron with TLS, static downloads, compression, and both MCP endpoints; verify before DNS cutover. See step 6.


- [ ] Deploy Alloy collection, MCP usage instrumentation, Grafana dashboards, synthetic checks, and alerts; measure overhead. See step 7.
^^^ need details, are we emit logs / metrics about app usage? I would like to have metrics for MCP server usage.
&&& Current `src/main.rs` initializes tracing output, but `src/server/mcp.rs` has no explicit tool-call counters, timing events, or usage instrumentation. Host metrics alone cannot answer MCP usage questions. Step 7 adds per-tool call counts, failures, and duration charts from completion events in Loki; HTTP access counts stay separate from tool calls.


- [ ] Document verified deployment commands, MCP endpoints, and the DigitalOcean retirement handoff. See step 8.

## Acceptance

- [ ] `nix flake check --no-build` passes in GitHub CI for the pinned configuration.
- [ ] `nix build .#nixosConfigurations.nixos-vps.config.system.build.toplevel --no-link` passes on the x86_64 Linux GitHub runner, including the application package.
- [ ] `ssh deploy@nixos-vps 'sudo -n true'` succeeds independently of the root session.
- [ ] `ssh deploy@nixos-vps 'systemctl is-active exodata ferron alloy'` reports all three services active.
- [ ] `curl --fail --silent --show-error https://exodata.space/ -o /dev/null` succeeds after cutover.
- [ ] `curl --fail --silent --show-error https://exodata.space/data/stellarhosts.parquet -o /dev/null` succeeds.
- [ ] `specdev check` and `git diff --check` pass; existing unrelated warnings are identified separately.

## Manual checks

- [ ] Deploy the exact system store path published by CI without Mac/VPS compilation; unavailable outputs cause deployment to fail. Production pages hydrate and lazy routes load; REST/MCP `health`, both TLS hostnames, WASM compression, and streaming work. Record commands, source revision, and store path.
- [ ] Activate the previous NixOS/application generation and restore the new generation; confirm application health after each activation and continued sudo SSH access.
- [ ] Grafana receives host metrics and application/proxy logs; known successful and failed MCP calls appear under the correct tool with duration and outcome, and usage charts match the calls made. A synthetic failure produces an alert at the verified destination. Record Alloy RSS and host memory before/after.

## Implementation details

### 1. Capture the existing Hostinger baseline

- Read `/etc/nixos/configuration.nix` and inspect the running system, disk labels, mount points, networking, SSH, cloud-init, and enabled services. Save the installed configuration as the provider baseline under `infrastructure/nix/`; do not regenerate it from a generic template.
- Preserve BIOS GRUB on `/dev/sda`, ext4 root at `/dev/disk/by-label/nixos`, partition/filesystem growth, serial console, cloud-init-managed networking with DHCP disabled, QEMU guest support, and the guest-agent PATH override.
- The QEMU guest agent runs inside the VPS so Hostinger can manage/provision it. It is unrelated to Mac virtualization. Preserve `system.stateVersion = "26.05"`; this is not the nixpkgs update selector.
- Create a root flake exposing `nixosConfigurations.nixos-vps` and lock nixpkgs to the revision of the installed system where obtainable. Keep provider settings separate from application modules. No `nix flake init`, reinstall, repartitioning, or local build is required.
- First checkpoint: CI evaluates/builds the baseline host only. Compare intended configuration with the installed system before activation; users, app, proxy, and monitoring are subsequent additions.
- Baseline files prepared: pinned flake, provider/shared/host modules, and `.github/workflows/nixos.yml`. Hostname remains `nixos`; VPS unchanged. CI evaluation/build awaits commit/push, so step 1 remains unchecked.

### 2. Administrator and service identities

- Create `deploy` as a normal user with the maintainer's selected SSH public key and membership permitting sudo. Verify `ssh deploy@nixos-vps` in a separate connection while root access remains available.
- Proposed activation policy: passwordless sudo for this administrator, matching `sudo -n` acceptance. This is effectively root authority after login; if password-prompted sudo is preferred, change activation and acceptance together before implementation.
- Only after login and privilege escalation work, disable password SSH authentication and restrict new root SSH logins. Keep provider-console recovery available and test reboot access before cutover.
- Give Exodata its own service identity with read-only access to runtime data/descriptions and no sudo. Proxy and Alloy use their own service accounts; Alloy receives only the journal/file access required for collection.

### 3. GitHub CI builds and published outputs

- Use an x86_64 Linux GitHub runner with Nix and the repository's `flake.lock`. CI runs `nix flake check --no-build`, then builds the host generation; once the app module is enabled, that build includes the application package.
- Define the application derivation with pinned Rust including the WASM target, cargo-leptos, Tailwind, native libraries, and Cargo.lock-based dependency vendoring. Run `cargo leptos build --release --split` inside the build and install the server plus the entire generated `site/` tree.
- Separate dependency preparation from source compilation so source-only changes can reuse dependency work. Validate the chosen Rust/WASM caching approach with a second source-only CI build; Nix alone does not guarantee incremental Cargo caching.
- Fetch standard packages from the public NixOS cache. Publish our application and system outputs with their runtime closure to the selected cache; keep compiler/build-only dependencies out of the deployed runtime closure.
- Publish a manifest containing Git revision, CI run, system store path, and cache location. Sign custom cache outputs and configure the VPS with the matching trusted public key. CI owns the upload/signing credentials; no secrets enter derivations or public logs.
- Neither the Mac nor VPS compiles anything. The first baseline build can run before app packaging is finished, so host adoption does not depend on completing the full release pipeline.

### 4. Cache retrieval, activation, and rollback

- Choose a cache service or object-storage binary cache before transport implementation; confirm access, cost, signing, and retention. This is the distribution point for CI outputs, not another build server.
- Proposed default: the Mac selects a successful CI manifest for a reviewed revision and sends an SSH command. The VPS imports that exact system path from the trusted cache with download-only behavior, then sudo activates it as a new system generation.
- Missing manifest, missing closure, signature failure, or revision mismatch aborts before activation. Do not invoke a build-capable fallback or silently deploy a newer cache entry. Document first-time cache trust/bootstrap separately.
- Alternative for review: manually triggered GitHub deployment using deploy-rs after the build, requiring an SSH credential and pinned host key in CI. deploy-rs remains optional; a laptop `deploy` invocation that could build locally does not meet the requirement.
- Just recipes will wrap CI triggering/inspection, manifest retrieval, SSH activation, status, journal logs, uploads, smoke checks, and rollback. Final recipe names and executable commands go into `DEPLOY.md` as they are implemented.
- Retain the current and previous system/application closures in both cache and host garbage-collection roots. Activation handles service changes; verify SSH and HTTP afterward. Select the previous generation to roll back, then test restoration of the new one. Kernel changes require a separately verified reboot.

### 5. Application service and mutable files

- Declare `systemd.services.exodata` with the packaged executable, restart policy, journald output, separate identity, and filesystem sandboxing. Bind only to `127.0.0.1:3000`; measure startup/prewarm memory before choosing `MemoryMax`.
- Set `LEPTOS_OUTPUT_NAME`, the package's absolute `LEPTOS_SITE_ROOT`, site address, `EXO_DATA_DIR`, and `EXO_CONTENT_DIR`. Read private configuration from a root-owned runtime file where needed, outside the Nix store.
- Upload `stellarhosts.parquet`, `exoplanets.parquet`, and both metadata TOML files, plus generated descriptions, to stable external directories. Validate completeness and ownership before restart; NixOS generation rollback does not restore changed data files.
- Check loopback pages, REST, MCP, description loading, and split hydration assets before exposing the host. Keep a compatible data snapshot during the migration and test service rollback before DNS changes.

### 6. Ferron, HTTPS, and public cutover

- Pin the selected Ferron version/module and validate its configuration against that version's documentation. Declare firewall access for SSH and HTTP/HTTPS; keep application and telemetry listeners private.
- Proxy `exodata.space` to loopback, expose only intended runtime download files under `/data/`, configure `/pkg/` caching and WASM compression, and allow startup/prewarm to complete before declaring health.
- Route `mcp.exodata.space/` internally to `/mcp`; preserve the main site's `/mcp`. Verify forwarded Host/Origin behavior against the app's MCP allowed-host checks rather than assuming a subdomain works automatically.
- Test real MCP initialization/tool calls, streaming/disconnect behavior, TLS for both hosts, static downloads, and compression. HTTP success alone is insufficient for MCP compatibility.
- Determine DNS ownership and certificate validation method. Use a temporary hostname or DNS validation if certificates cannot be issued before public records move; verify the new host using explicit address resolution, then change DNS and recheck externally. Keep DigitalOcean available during propagation.

### 7. Host, HTTP, and MCP observability

- Alloy collects host CPU/load, RAM/swap, disk, and network metrics through its Unix exporter and forwards them to Grafana Cloud metrics. It ships Exodata's journal plus proxy access/error logs to Loki; pin collection settings to the installed service versions.
- Host metrics describe server resources. Proxy logs describe HTTP request counts, status, bytes, and latency where available, grouped by bounded route families such as website, REST, and MCP. They do not identify tool calls or establish unique human users.
- Add one structured completion event per dispatched MCP tool invocation with a fixed event name, tool name, success/error outcome, and elapsed duration. Cover all six tools and error paths; count tool/protocol errors independently of HTTP 200 responses. Identify validation failures rejected before handler entry separately if observable.
- Proposed first implementation uses these events in Loki to chart calls per tool, error rate, and duration distributions over time; this provides usage measurements without adding a Prometheus endpoint. An exact-once durable audit count is not promised by log shipping. Separate synthetic health traffic from ordinary tool usage in charts.
- Keep labels bounded to service, environment, tool, and outcome. Do not collect SQL text, tool arguments/results, tokens, client IPs, session IDs, or arbitrary URLs as metric labels; configure proxy logs to omit query strings and unwanted identifying fields.
- Supply Grafana endpoints and credentials via protected runtime files. Configure node-resource, HTTP, and MCP dashboards, service/error/resource alerts, and external uptime checks with a verified notification destination.
- Verify a known batch of successful and failed calls against Grafana tool counts and durations, plus malformed input behavior and an alert test. Measure Alloy RSS and collection overhead. Relevant references: [Alloy journal source](https://grafana.com/docs/alloy/latest/reference/components/loki/loki.source.journal/) and [Ferron observability](https://ferron.sh/docs/v2/use-cases/logging-observability); confirm Ferron documentation version during pinning.

### 8. Operator documentation and handoff

- Update `DEPLOY.md` with the exact initial bootstrap, CI trigger, artifact selection, cache trust, activation, data/prose upload, health, logs, rollback, and reboot commands verified during this task. State which machine runs each command.
- Update MCP connection examples only after the subdomain is verified; document continuing support for the existing `/mcp` URL. Keep credentials and mutable runtime files out of repository examples.
- Record deployed source revision/system path, verification evidence, remaining prerequisites, and the old-host recovery window. The maintainer decides when to delete DigitalOcean; this task does not destroy it or remove its deployment tooling.

## Context

- Original requested completion: Saturday 2026-11-07; work remains in planning as of 2026-11-08, with no replacement deadline agreed. This is an umbrella deployment task (`huge`) for the requested complete migration; its host, build, proxy, and monitoring steps share a cutover and rollback acceptance gate.
- General direction: [infrastructure.md](../infrastructure.md).
- Inspected over `ssh root@nixos-vps`: NixOS 26.05, x86_64, one CPU, 3916 MiB RAM, approximately 46 GB free, no swap, only SSH exposed on TCP. No `deploy` user or running nginx/Ferron/Alloy service was found. No remote configuration has been changed.
- Reference: [dreamsofcode-io/nixos-vps](https://github.com/dreamsofcode-io/nixos-vps), whose provider module closely matches the installed template. Borrow modular organization; do not adopt default local builds or early root-login restriction. Its `sops-nix` secrets approach remains optional.
- Prerequisites: confirm DNS provider/access and Grafana credential locations; verify contact forwarding before using `acme@` or `alerts@`. Never put tokens in task evidence.
- Preserve existing production until replacement checks pass. The maintainer destroys the DigitalOcean instance separately; do not run OpenTofu destroy or remove old deployment tooling in this task.
- Git staging, commits, and pushes remain user-managed. The CI run must use the reviewed source revision published by the maintainer.

## Summary

Migrate production to Hostinger NixOS with CI builds, native services, Ferron, MCP usage monitoring, and verified rollback.

## Log

- 2026-11-07 created: hostinger nixos deployment
- 2026-11-07 set grade huge
- 2026-11-07 scope add flake.nix — Complete NixOS deployment: pinned package and host modules, CI, operator commands, runbook, and MCP endpoint documentation
- 2026-11-07 scope add flake.lock — Complete NixOS deployment: pinned package and host modules, CI, operator commands, runbook, and MCP endpoint documentation
- 2026-11-07 scope add infrastructure/nix/** — Complete NixOS deployment: pinned package and host modules, CI, operator commands, runbook, and MCP endpoint documentation
- 2026-11-07 scope add .github/workflows/deploy.yml — Complete NixOS deployment: pinned package and host modules, CI, operator commands, runbook, and MCP endpoint documentation
- 2026-11-07 scope add Justfile — Complete NixOS deployment: pinned package and host modules, CI, operator commands, runbook, and MCP endpoint documentation
- 2026-11-07 scope add DEPLOY.md — Complete NixOS deployment: pinned package and host modules, CI, operator commands, runbook, and MCP endpoint documentation
- 2026-11-07 scope add docs/mcp.md — Complete NixOS deployment: pinned package and host modules, CI, operator commands, runbook, and MCP endpoint documentation
- 2026-11-08 scope add src/server/mcp.rs — User requested MCP usage metrics; add tool invocation outcome and duration events for Grafana usage charts
- 2026-11-08 scope add .github/workflows/nixos.yml — Approved step-1 baseline CI workflow
