---
id: 0017-hostinger-nixos-deployment
status: draft
scope: [flake.nix, flake.lock, infrastructure/nix/**, .github/workflows/deploy.yml, Justfile, DEPLOY.md, docs/mcp.md, src/server/mcp.rs, .github/workflows/nixos.yml]
created: 2026-11-07
grade: huge
---
# Task: hostinger nixos deployment

## Plan

- [x] Capture the installed Hostinger baseline in a pinned flake and add baseline CI. See step 1; files reviewed by user, CI execution pending.

- [ ] Manage the server as root over the existing SSH access; keep login settings as installed and keep secrets out of Git and the Nix store. See step 2.

- [ ] Build the application and NixOS generation with Nix in GitHub CI and push outputs to the public Cachix cache `exodata`. See step 3.

- [ ] Deploy the exact CI output from the Mac over root SSH with download-only activation on the VPS; add operator recipes. See step 4.


- [ ] Declare the application service, upload external data/descriptions, and verify startup and rollback before cutover. See step 5.


- [ ] Deploy Ferron with TLS, static downloads, compression, and both MCP endpoints; verify before DNS cutover. See step 6.


- [ ] Deploy Alloy collection, MCP usage instrumentation, Grafana dashboards, synthetic checks, and alerts; measure overhead. See step 7.


- [ ] Document verified deployment commands, MCP endpoints, and the DigitalOcean retirement handoff. See step 8.

## Acceptance

- [ ] `nix flake check --no-build` passes in GitHub CI for the pinned configuration.
- [ ] `nix build .#nixosConfigurations.exodata-vps.config.system.build.toplevel --no-link` passes on the x86_64 Linux GitHub runner, including the application package.
- [ ] `ssh root@exodata-vps 'systemctl is-active exodata ferron alloy'` reports all three services active.
- [ ] `curl --fail --silent --show-error https://exodata.space/ -o /dev/null` succeeds after cutover.
- [ ] `curl --fail --silent --show-error https://exodata.space/data/stellarhosts.parquet -o /dev/null` succeeds.
- [ ] `specdev check` and `git diff --check` pass; existing unrelated warnings are identified separately.

## Manual checks

- [ ] Deploy the exact system store path published by CI without Mac/VPS compilation; unavailable outputs cause deployment to fail. Production pages hydrate and lazy routes load; REST/MCP `health`, both TLS hostnames, WASM compression, and streaming work. Record commands, source revision, and store path.
- [ ] Activate the previous NixOS/application generation and restore the new generation; confirm application health after each activation and continued root SSH access.
- [ ] Grafana receives host metrics and application/proxy logs; known successful and failed MCP calls appear under the correct tool with duration and outcome, and usage charts match the calls made. A synthetic failure produces an alert at the verified destination. Record Alloy RSS and host memory before/after.

## Implementation details

### 1. Capture the existing Hostinger baseline

- Captured `/etc/nixos/configuration.nix` into provider, shared SSH, and host modules under `infrastructure/nix/`. Preserved BIOS GRUB on `/dev/sda`, labelled ext4 root, filesystem growth, serial console, cloud-init networking with DHCP disabled, QEMU guest support, provisioning tools, and agent PATH.
- Root flake exposes `nixosConfigurations.exodata-vps`, pins installed nixpkgs revision `08d3a97586f6b6343b3b5e799980773629432610`, and retains `system.stateVersion = "26.05"`. After capture, the user renamed the target, SSH alias, and hostname from `nixos-vps`/`nixos` to `exodata-vps`; the hostname changes at first activation. Provider settings are separate from reusable modules; no reinstall was needed.
- QEMU guest agent is Hostinger's in-VPS provisioning helper, unrelated to Mac virtualization.
- Added `.github/workflows/nixos.yml` for Linux-only baseline evaluation/build. Local Nix parsing and diff checks passed; no Mac/VPS build or remote configuration change occurred.
- User accepted the baseline files and requested step 1 marked done. CI evaluation/build still awaits commit/push and remains an unticked acceptance gate; compare the evaluated baseline before activation.

### 2. SSH access and secrets

- Decision (2026-11-09): administer the server as root over SSH with the maintainer's key already in `/root/.ssh/authorized_keys`. No separate administrator user, no sudo policy, no SOPS. SSH login settings stay as installed (`PermitRootLogin = "yes"`, other settings untouched); any hardening is a separate later change.
- No keys, tokens, or contact emails in Git or the Nix store (the Cachix cache is public). Runtime values such as the ACME/alert address or Grafana token are placed by hand in root-owned files on the server and read by services at runtime (e.g. systemd `EnvironmentFile`).
- Evaluate only through Git-backed flake refs (`.#…`, `github:…`). `path:` refs copy Git-ignored files, including legacy tofu/ansible credentials, into the world-readable local Nix store.
- Removed the superseded approach: sops-nix input, `modules/administrator.nix`, `bootstrap-secrets.sh`, and administrator CI assertions. Target renamed to `exodata-vps` (flake output, host file, SSH alias, hostname). VPS unchanged.

### 3. GitHub CI builds and published outputs

- Use an x86_64 Linux GitHub runner with Nix and the repository's `flake.lock`. CI runs `nix flake check --no-build`, then builds the host generation; once the app module is enabled, that build includes the application package.
- Define the application derivation with pinned Rust including the WASM target, cargo-leptos, Tailwind, native libraries, and Cargo.lock-based dependency vendoring. Run `cargo leptos build --release --split` inside the build and install the server plus the entire generated `site/` tree.
- Separate dependency preparation from source compilation so source-only changes can reuse dependency work. Validate the chosen Rust/WASM caching approach with a second source-only CI build; Nix alone does not guarantee incremental Cargo caching.
- Fetch standard packages from `cache.nixos.org`. CI pushes our application and system outputs to the public Cachix cache `exodata` (`https://exodata.cachix.org`, Cachix-managed signing, public key `exodata.cachix.org-1:pgrrFg4Ejd8QJWtEpXwV2U3tXKaDtwnGFI7s05BLQiU=`) via `cachix/cachix-action` with GitHub secret `CACHIX_AUTH_TOKEN` (created by the maintainer). Keep compiler/build-only dependencies out of the deployed runtime closure.
- The deployed revision is the Git commit; CI's successful build of that commit defines the system store path. No secrets enter derivations or public logs.
- No Docker on the NixOS host. The existing Docker/GHCR workflow keeps serving DigitalOcean until its retirement.
- Neither the Mac nor VPS compiles anything. The first baseline build can run before app packaging is finished, so host adoption does not depend on completing the full release pipeline.

### 4. Cache retrieval, activation, and rollback

- VPS configuration trusts `exodata.cachix.org` (`nix.settings.substituters` and `trusted-public-keys`) and sets `nix.settings.max-jobs = 0`, so it never builds: an output missing from the caches fails the deploy instead of compiling.
- The installed system does not trust the cache yet; the first activation passes the cache as `--option extra-substituters` / `extra-trusted-public-keys` on the command line.
- Proposed deploy, from the Mac after CI succeeds for commit `<rev>`: `ssh root@exodata-vps 'nixos-rebuild switch --flake github:oiwn/exodata/<rev>#exodata-vps'`. deploy-rs (`remoteBuild = true`, run from the Mac) remains an optional later addition for automatic rollback. CI holds no SSH credential to the server.
- Just recipes will wrap CI triggering/inspection, manifest retrieval, SSH activation, status, journal logs, uploads, smoke checks, and rollback. Final recipe names and executable commands go into `DEPLOY.md` as they are implemented.
- Retain the current and previous system/application closures in both cache and host garbage-collection roots. Activation handles service changes; verify SSH and HTTP afterward. Select the previous generation to roll back, then test restoration of the new one. Kernel changes require a separately verified reboot.

### 5. Application service and mutable files

- Declare `systemd.services.exodata` with the packaged executable, restart policy, journald output, separate identity, and filesystem sandboxing. Bind only to `127.0.0.1:3000`; measure startup/prewarm memory before choosing `MemoryMax`.
- Set `LEPTOS_OUTPUT_NAME`, the package's absolute `LEPTOS_SITE_ROOT`, site address, `EXO_DATA_DIR`, and `EXO_CONTENT_DIR`. Read private configuration from a root-owned runtime file where needed, outside the Nix store.
- Give Exodata its own service identity with read-only access to runtime data/descriptions and no sudo. Proxy and Alloy use their own service accounts; Alloy receives only the journal/file access required for collection.
- Upload from the Mac with rsync over root SSH: `stellarhosts.parquet`, `exoplanets.parquet`, and both metadata TOML files to `/var/lib/exodata/data`; generated descriptions to `/var/lib/exodata/content` (same layout as the current `upload-descriptions.yml`). The NixOS configuration creates these directories; data stays outside Nix. Validate completeness and ownership before restart; NixOS generation rollback does not restore changed data files.
- Check loopback pages, REST, MCP, description loading, and split hydration assets before exposing the host. Keep a compatible data snapshot during the migration and test service rollback before DNS changes.

### 6. Ferron, HTTPS, and public cutover

- Pin the selected Ferron version/module and validate its configuration against that version's documentation. Declare firewall access for SSH and HTTP/HTTPS; keep application and telemetry listeners private.
- Proxy `exodata.space` to loopback, expose only intended runtime download files under `/data/`, configure `/pkg/` caching and WASM compression, and allow startup/prewarm to complete before declaring health.
- Route `mcp.exodata.space/` internally to `/mcp`; preserve the main site's `/mcp`. Verify forwarded Host/Origin behavior against the app's MCP allowed-host checks rather than assuming a subdomain works automatically.
- Test real MCP initialization/tool calls, streaming/disconnect behavior, TLS for both hosts, static downloads, and compression. HTTP success alone is insufficient for MCP compatibility.
- Determine DNS ownership and certificate validation method. Use a temporary hostname or DNS validation if certificates cannot be issued before public records move; verify the new host using explicit address resolution, then change DNS and recheck externally. Keep DigitalOcean available during propagation.

### 7. Host, HTTP, and MCP observability

- Current tracing output covers operational logs; MCP has no explicit per-tool usage/timing instrumentation. Add it as part of this step.
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

- Original target was 2026-11-07; no replacement deadline agreed. Baseline files are implemented and reviewed; deployment remains pending. This umbrella task (`huge`) shares a cutover and rollback acceptance gate across host, build, proxy, and monitoring work.
- General direction: [infrastructure.md](../infrastructure.md).
- Inspected over `ssh root@nixos-vps`: NixOS 26.05, x86_64, one CPU, 3916 MiB RAM, approximately 46 GB free, no swap, only SSH exposed on TCP. No `deploy` user or running nginx/Ferron/Alloy service was found. No remote configuration has been changed.
- Reference: [dreamsofcode-io/nixos-vps](https://github.com/dreamsofcode-io/nixos-vps). Adopt modular organization; avoid its SOPS bootstrap, default local builds, and root-login restriction.
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