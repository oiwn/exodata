# AGENTS.md

## Purpose
This file defines agent workflow and points to project specifications. Implementation details live in `specs/` (not here).

## Read First
- `specs/overview.md` — project status, entry points, and links to all specs
- `specs/ctx.md` — current priorities and near-term plans
- `specs/web-frontend.md` — UI/Leptos requirements
- `specs/web-backend.md` — Axum/server requirements
- `specs/cli.md` — CLI requirements
- `specs/data-management.md` — data fetching/prep
- `specs/column-metadata.md` — column descriptions/units
- `specs/problems.md` — known issues/edge cases
- `DEPLOY.md` and `README.md` — build/test/deploy commands

## Workflow Guidelines
1. Prefer specifications over code when behavior is unclear.
2. If specs and code diverge, ask for clarification or update specs before changing behavior.
3. Keep specs purely technical (implementation details, interfaces, expected outputs). Avoid scientific/astrophysics explanations.
4. When adding a new subsystem or feature, add/update the relevant spec in `specs/`.
5. Keep AGENTS.md high-level; do not duplicate implementation details from specs.

## Agent Rules
1. **Explicit Instruction Compliance:** Do not perform actions (file edits or command execution) without explicit user request.
2. **Confidence Threshold:** If below ~70% confidence about a request or outcome, stop and ask for clarification.
