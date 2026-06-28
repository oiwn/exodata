# AGENTS.md

## Purpose
This file defines agent workflow and points to project specifications. Implementation details live in `specs/` (not here).

## Read First
- `specs/overview.md` — project status, entry points, and links to all specs
- `specs/ctx.md` — current task context / working notes; treat it as temporary context for the active task and clean it up when the task is finished and the content is no longer needed

## Workflow Guidelines
1. Prefer specifications over code when behavior is unclear.
2. If specs and code diverge, ask for clarification or update specs before changing behavior.
3. Keep specs purely technical (implementation details, interfaces, expected outputs). Avoid scientific/astrophysics explanations.
4. When adding a new subsystem or feature, add/update the relevant spec in `specs/`.
5. Keep AGENTS.md high-level; do not duplicate implementation details from specs.

## Agent Rules
1. **Explicit Instruction Compliance:** Do not perform actions (file edits or command execution) without explicit user request.
2. **Confidence Threshold:** If below ~70% confidence about a request or outcome, stop and ask for clarification.
3. **Response Style Preference:** Prefer single-view answers; if possible, keep responses short enough to fit on one page.
4. **Fixtures Are Not Source of Truth:** Treat test fixtures as stable sample material for testing only. Do not infer or enforce behavior from exact fixture numbers when those numbers only reflect a sampled snapshot of the live dataset.

## Behavioral Guidelines

These guidelines bias toward caution over speed. For trivial tasks, use judgment.

### Think Before Coding

Do not assume or hide confusion. Surface assumptions and tradeoffs before implementing.

- State assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them instead of choosing silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop, name what is confusing, and ask.

### Simplicity First

Write the minimum code that solves the requested problem.

- Do not add features beyond what was asked.
- Do not add abstractions for single-use code.
- Do not add flexibility or configurability that was not requested.
- Do not add error handling for impossible scenarios.
- If a change is becoming much larger than necessary, simplify before continuing.

### Surgical Changes

Touch only what the request requires. Clean up only issues introduced by the current change.

- Do not improve adjacent code, comments, or formatting opportunistically.
- Do not refactor unrelated code.
- Match existing style, even when another style seems preferable.
- If unrelated dead code is noticed, mention it instead of deleting it.
- Remove imports, variables, or functions only when the current change made them unused.
- Every changed line should trace directly to the user's request.

### Goal-Driven Execution

Turn implementation tasks into verifiable goals.

- For bug fixes, prefer a reproducing test before the fix when practical.
- For validation changes, test invalid inputs and expected failures.
- For refactors, verify behavior before and after when feasible.
- For multi-step tasks, state a brief plan with a verification check for each step.
- Prefer the fastest meaningful manual check once a feature is runnable. If the
  manual check fails, stop broad test/diff loops and fix the observed behavior
  directly.
- Once a change compiles, do not start additional broad verification loops before
  the fastest meaningful manual check. If the user is expected to run the app or
  inspect the UI manually, stop and hand them the exact command/URL/checklist
  instead of continuing with more automated checks.

Strong success criteria allow independent progress. Weak criteria require clarification before coding.
