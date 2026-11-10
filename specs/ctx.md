# Current Task Context

Current focus: [0017-hostinger-nixos-deployment](tasks/0017-hostinger-nixos-deployment.md) (draft), step 4. Decisions recorded in its steps 2–5.

## catchup

- Root SSH as installed; no deploy user, no SOPS. Target is `exodata-vps`.
- Public Cachix cache `exodata`; `CACHIX_AUTH_TOKEN` is in GitHub secrets.
- Step 3 done: `nixos.yml` (manual dispatch) builds `.#exodata` and the host config and pushes both to Cachix.
- Step 4 written, not yet deployed: `modules/base.nix` Nix settings; `just nix-deploy <rev>` / `just nix-rollback`. VPS still runs the installed system (flakes disabled).
- Never evaluate with `path:.`.

## Next

1. User: commit, push to `dev`, `gh workflow run nixos.yml --ref dev`, wait for green.
2. User: `just nix-deploy <that commit>`; check SSH still works and `nix config show substituters` on the VPS lists Cachix.
3. Step 5: systemd service, `/var/lib/exodata/{data,content}`, rsync upload recipes.
