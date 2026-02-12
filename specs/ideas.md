# Random ideas about project

- [ ] would be cool to route to api endpoints if i just add ".json" at the end of address.
- [ ] hover for table column with description
- [ ] need mapping between field names and their short description
- [ ] downloader and diffs for data extracted from caltech
- [ ] table legend for err/lim indicators + collapsible columns list with descriptions (exclude err1/err2/lim)
- [ ] REST endpoint for exoplanet detail by name (group duplicate rows)
- [ ] cache cleanup strategies

## Doubts
- [ ] shared table page state + query encode/decode (page/sort/columns/filters)
- [ ] column metadata duplication (single shared type or feature-gated exo-core)
- [ ] dataframe_to_json dtype coverage + more efficient iteration
- [ ] proper URL encoding for link column in table
- [ ] reusable pagination controls + shared loading/error UI
- [ ] column selector text says "drag to reorder" but only up/down buttons
- [ ] SelectedColumnsList uses idx key; use column name
- [ ] confirm/remove unused `src/stellarhosts.rs` + `src/common.rs`
- [ ] tests: mirror exoplanets coverage + query encode/decode + dtype handling
- [ ] decide filter URL format + filter semantics
