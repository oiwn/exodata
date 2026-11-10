# Current Task Context

Current focus: [0017-hostinger-nixos-deployment](tasks/0017-hostinger-nixos-deployment.md) (draft), step 3. Decisions recorded in its steps 2–5.

## catchup

- Root SSH as installed; no deploy user, no SOPS. Target is `exodata-vps`. VPS unchanged.
- Public Cachix cache `exodata`; `CACHIX_AUTH_TOKEN` is in GitHub secrets.
- App package `infrastructure/nix/packages/exodata.nix` (crane + rust-overlay) evaluates for x86_64-linux; not yet built. `wasm-bindgen-cli` vendor hash is still `lib.fakeHash`.
- Never evaluate with `path:.`. Mac-side evaluation of uncommitted work uses a scratch Git repo of tracked files only.

## Next

1. User: stage `infrastructure/nix/packages/exodata.nix`, `flake.nix`, `flake.lock`, workflow, specs; commit; push to `dev`.
2. Read the CI log: replace the fake vendor hash with the reported one, then fix any build errors until `.#exodata` builds and is pushed to Cachix.
