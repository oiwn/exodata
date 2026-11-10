# Current Task Context

Current focus: [0017-hostinger-nixos-deployment](tasks/0017-hostinger-nixos-deployment.md) (draft). Decisions recorded in its steps 2–5.

## catchup

- Root SSH as installed; no deploy user, no SOPS. Target is `exodata-vps`; user renames `Host nixos-vps` in `~/.ssh/config`.
- Public Cachix cache `exodata` exists; `CACHIX_AUTH_TOKEN` is in GitHub secrets.
- Never evaluate with `path:.` (copies ignored credential files into the Nix store). An older such copy may remain at `/nix/store/0r7ry5i8bpwmpl4iswig75pwmzaxqq8g-source`; user decides on removal.
- VPS unchanged.

## Next

1. User: stage changes (rename, deletions), delete untracked `infrastructure/nix/secrets/`, push; confirm `NixOS baseline` CI passes.
2. Step 3: Nix package for the app (Rust + WASM + site) built in CI and pushed to Cachix.
