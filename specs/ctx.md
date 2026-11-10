# Current Task Context

Current focus: [0017-hostinger-nixos-deployment](tasks/0017-hostinger-nixos-deployment.md) (draft), step 6. Decisions recorded in its steps 2–6.

## catchup

- Root SSH as installed; no deploy user, no SOPS. Target is `exodata-vps` (194.113.64.241).
- CI (`nixos.yml`, manual dispatch) builds the app and host and pushes them to Cachix. `just nix-deploy <rev>` / `nix-rollback` / `nix-upload-data` / `nix-upload-descriptions`.
- Steps 4–5 deployed: the app ran publicly on :3000.
- Step 6 written, not deployed: Ferron (`modules/ferron.nix`) on 80/443 with LE HTTP-01; the app moved to 127.0.0.1:3000; `mcp.exodata.space` gets a Host rewrite.
- DNS: Cloudflare, proxied, Full (strict). DO stays up until the maintainer removes it.
- Never evaluate with `path:.`.

## Next

1. User: commit, push to `dev`, run CI, then `just nix-deploy <rev>`. Then `systemctl is-active ferron exodata`.
2. Cutover per spec step 6: grey-cloud A records → wait for certificates → orange cloud.
3. Verify `/data/`, wasm compression, MCP on both hosts, port 3000 closed, ACME renewal path.
4. Step 7: Alloy.
