# Current Task Context

Current focus: [0017-hostinger-nixos-deployment](tasks/0017-hostinger-nixos-deployment.md) (draft), step 7. Decisions recorded in its steps 2–7.

## catchup

- Live on the NixOS VPS (194.113.64.241): Ferron + LE certificates behind the Cloudflare proxy (Full strict); MCP at `mcp.exodata.space` (`/mcp` returns 308). DO is still up.
- Deploy: CI `nixos.yml` (manual) → `just nix-deploy <rev>`. Data: `just nix-upload-data` / `nix-upload-descriptions`.
- Step 7 written, not deployed: Alloy (`modules/alloy.nix`), IP/SQL-free Ferron log, MCP `mcp_tool_call` events. Secrets: `just nix-push-secrets` from the Git-ignored `infrastructure/nix/alloy.env`.
- Never evaluate with `path:.`. The agent must not read `*.env` files.

## Next

1. User: fill in `alloy.env`, `git check-ignore -v` it, `just nix-push-secrets`, commit/push, CI, `just nix-deploy`.
2. Verify Alloy has no auth errors; Explore shows `node_load1{instance="exodata-vps"}` and `{unit="ferron.service"}`; test MCP calls appear.
3. Grafana UI: Linux Server integration dashboards, MCP panels, alerts, synthetic checks.
4. Step 8: DEPLOY.md, handoff.
