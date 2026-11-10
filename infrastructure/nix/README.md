# NixOS host

`nixosConfigurations.exodata-vps` describes the Hostinger VPS. Builds run only in GitHub CI; the Mac and the VPS never compile.

- Access: root over SSH with the key already on the server. Login settings are kept as installed.
- No secrets in this directory, Git, or the Nix store. Runtime values (contact emails, tokens) are placed by hand in root-owned files on the server.
- Evaluate with Git-backed refs only (`.#exodata-vps`). Never use `path:.` here: it copies Git-ignored credential files into the local Nix store.

Deployment, cache, and data upload commands are added here as they are implemented (task 0017, steps 3–5).
