# Current Task Context: Open-issue triage and next-session plan

State: in progress

## Plan

- [ ] Review all 33 open issues with the user one at a time; decide whether to close, clarify, implement, investigate, or defer each issue. Do not mutate GitHub without explicit approval.
- [ ] Start with closure candidates #10, #36, #57, #61, #83, and #107, recording the evidence/comment needed before any closure.
- [ ] Verify tmux italic fix (new pane + reattach), then optionally confirm matrix theme renders dim green slanted thinking text.

## Findings

GitHub review on 2026-08-27 found 33 open issues in `oiwn/exodata`; none had comments. No issue was edited or closed.

- **Closure candidates:** #10 (homepage statistics and graphical distribution bars satisfy the aggregation/visualization intent); #36 (live `/pkg/exodata.wasm` returns `Cache-Control: no-cache`); #57 (stellar-host planet cards link to planet details); #61 (host provenance table exposes measurements and references); #83 (homepage Exoplanets heading uses `animate-pulse`); #107 (overview classifies canonical median radii into five size categories).
^^^ no there must be separate page with graphics i think we'll use "rust-ui"
- **Valid, still actionable:** #133 (no OpenCode CI workflow; acceptance criteria vague); #131 (no Pi-harness setup; underspecified); #129 (footer still uses long utility-class strings); #119 (top/manual target remains `#mcp-exoplanet-data`, not `#manual`); #116 (no AI-generated detail reasoning); #115 (`<main>` exists on overview/docs but is absent from tables, insights, detail, and error pages); #111 (crates.io-facing `crates/exo-cli/README.md` still advertises MCP); #108 (no routing progress bar); #104 (no URL shortener); #100 (no arXiv integration); #98 (no `llms.txt` file/route); #95 (missing recent planets and largest/smallest host insights); #82 (API/CLI are links/cards, not tabs); #75 (404 has no facts/insights links); #69 (live invalid detail still renders `Error Loading Planet`, not branded 404); #45 (filter is case-insensitive substring search; no strict mode); #27 (prewarm uses `selected_columns: None`, unlike explicit SSR fetch columns); #11 (2025 appears in an overview bar, but there is no dedicated graphical page).
- **Partially implemented:** #99 (hero/comparison use medians, but quick-summary cards still use `records.first()`); #87 (nginx declares gzip/WASM support, but render-blocking CSS/analytics remain and live CSS/JS still report four-hour caching); #77 (binary-systems insight exists, requested host-detail visualization does not); #59 (detail pages have JSON/CSV downloads, but general REST/API links are absent); #58 (detail references/exports exist, but wide-table reference links, presets, copy/share, and short URLs remain incomplete).
- **Investigate or resolve as non-code:** #97 (no `signature-agent` injection in this tree; needs browser/network reproduction); #90 (ratio formula and units are consistent, with values above one caused by compact hosts such as white dwarfs; scientific/data decision); #72 (local data has consistent `1195.98 pc` distance and `8868.67-29300 K` temperatures from different references; provenance is exposed, scientific acceptance remains); #43 (nginx remains deployed; Ferron is an infrastructure research choice, not a defect).

## Context

Use the GitHub issue body plus current code as the source of truth for each review. Separate implementation evidence from production evidence and scientific/product judgment. Report first; never close, comment on, or edit issues unless the user explicitly requests that mutation.

## Next

Continue step by step with closure candidate #36, then #57, #61, #83, #107, and #10 unless the user selects another issue.
