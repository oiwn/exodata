# Current Task Context: Open-issue triage and next-session plan

State: in progress

## Plan

- [ ] Provide a manual closure verification list for #57, #75, #83, #95, #107, #36, and #27; do not close the issues automatically.
- [ ] Verify tmux italic fix (new pane + reattach), then optionally confirm matrix theme renders dim green slanted thinking text.

## Findings

- **Closure candidates:** #57 is represented by clickable planet names in stellar-host detail cards; #83 by the animated homepage Exoplanets heading; #107 by the radius-based planet classifications on the overview.
- **Do not close yet:** #75 still has no Facts/Insights link on the 404 page; #95 still lacks the requested recently detected exoplanets and largest/smallest stellar-host rankings; #27 still prewarms `selected_columns: None` while SSR table requests use explicit display plus uncertainty/limit columns, producing different cache keys.
- **Needs production verification:** #36 is addressed in deployment configuration by sending `Cache-Control: no-cache` for `/pkg/`, but verify the live JS/WASM response headers after deployment before closing.
- **Partially covered / still requires scoped work:** #58 (reference links and exports have backend support, but no column presets or share control); #87 (gzip/WASM support and cache headers were addressed, but GA remains render-path work); #99 (detail canonicalization exists, but the reported record-selection case needs a reproducible rule); #59 (API/export links exist in the manual, not clearly on table/detail pages); #69 (route fallback has a branded 404, but detail lookup errors still surface as loading errors).
- **Research or product decisions, not next-session implementation:** #10, #11, #43, #72, #77, #90, #100, #104, #108, #116. #97 needs production browser/network evidence because no `signature-agent` injection is in this tree. #111 is ambiguous: the product intentionally documents a hosted MCP server, so define whether removal is limited to the CLI package/docs before changing copy. #45 strict search needs a filter-semantics decision.

## Context

Verify the implemented issue candidates against their original acceptance criteria and provide the user with URLs or commands for manual closure.

## Next

Prepare the manual closure verification list for the implemented issue candidates.
