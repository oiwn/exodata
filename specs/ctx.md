# Current Task Context: Rename repository identity and deploy

State: in progress

## Plan

- [x] Rename repository, package metadata, operational identifiers, and public links to Exodata.
- [x] Keep CI as build-and-push only; deployment stays manual via Ansible from local.
- [ ] Bump version, merge to `main`, and let CI build & push the `ghcr.io/oiwn/exodata` image.
- [ ] Deploy with `just ansible-deploy` locally and verify production.

## Next

Bump the version in `Cargo.toml`, open a PR and merge to `main`. CI builds the
image (website + wasm) and pushes it to GHCR as `exodata:<version>` + `latest`.
Then run `just ansible-deploy` locally (pulls `latest`) and verify the deployed
health endpoint at https://exodata.space.
