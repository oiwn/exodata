# Current Task Context

Current focus: [0017-hostinger-nixos-deployment](tasks/0017-hostinger-nixos-deployment.md) (draft), steps 7–8. Decisions recorded in its steps 2–7.

## catchup

- Live on the NixOS VPS (194.113.64.241): Ferron + LE certificates behind the Cloudflare proxy (Full strict). MCP is at `mcp.exodata.space` (`/mcp` returns 308). DO is still up.
- CI `nixos.yml` runs on `v*` tags or by hand and pushes to Cachix; it never deploys. Deploy: `just nix-deploy <rev>`. Data: `just nix-upload-data` / `nix-upload-descriptions`. Secrets: `just nix-push-secrets`.
- Step 7 is deployed: Alloy sends host metrics and journal logs to Grafana Cloud (no errors). The Ferron log has no IPs or query strings. MCP `mcp_tool_call` events are verified.
- Never evaluate with `path:.`. The agent must not read `*.env` files.

## Next

1. User: PR `dev` → `main`, merge; then `git tag vX.Y.Z && git push origin vX.Y.Z` to confirm the tag-triggered build.
2. Grafana: the dashboard in `tmp/exodata-vps-dashboard.json` (v2 format, default data sources) has empty Ferron panels. Check the data source and the `{unit="ferron.service"}` lines. Add alerts and uptime checks.
3. Step 8: DEPLOY.md, handoff, and the DO shutdown decision.
