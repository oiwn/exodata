# Current Task Context: Open-issue triage and next-session plan

State: in progress

## Plan

- [ ] Add a concise explanation of stellar spectral-class letters to the overview’s Stellar Classes card, matching the guidance style of Planet Classifications.
- [ ] Provide a manual closure verification list for #57, #75, #83, #95, #107, #36, and #27; do not close the issues automatically.

## Findings

- **Stellar Classes follow-up:** the distribution is visually clear, but readers need a short in-card explanation of what classes such as G, K, and M represent; match the existing Planet Classifications treatment.
- **Partially covered / still requires scoped work:** #58 (reference links and exports have backend support, but no column presets or share control); #87 (gzip/WASM support and cache headers were addressed, but GA remains render-path work); #99 (detail canonicalization exists, but the reported record-selection case needs a reproducible rule); #59 (API/export links exist in the manual, not clearly on table/detail pages); #69 (route fallback has a branded 404, but detail lookup errors still surface as loading errors).
- **Research or product decisions, not next-session implementation:** #10, #11, #43, #72, #77, #90, #100, #104, #108, #116. #97 needs production browser/network evidence because no `signature-agent` injection is in this tree. #111 is ambiguous: the product intentionally documents a hosted MCP server, so define whether removal is limited to the CLI package/docs before changing copy. #45 strict search needs a filter-semantics decision.

## Context

Verify the implemented issue candidates against their original acceptance criteria and provide the user with URLs or commands for manual closure.

## Next

Implement the Stellar Classes explanation.
