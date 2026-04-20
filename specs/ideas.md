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

## TODO:

- [ ] add skill for agent how to use cli tool.
- [ ] MCP server

## Insights page

- smallest exoplanets by radius
- largest exoplanets by radius
- hottest exoplanets by equilibrium temperature
- coldest exoplanets by equilibrium temperature
- nearest stellar hosts
- hottest stellar hosts
- coolest stellar hosts
- most massive stellar hosts
- stellar hosts with the most planets
- binary planetary systems with planets
- planetary systems with the most planets
- most equal star-planet pairs
- stellar hosts with the largest known planet
- compact systems with the shortest orbital periods
- largest planet-to-host size ratios
- hottest planets around the coolest stars
- nearest systems with multiple known planets
- densest small exoplanets
- lowest-density giant exoplanets
- oldest stellar hosts with planets

