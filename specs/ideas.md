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

- [ ] short url so long list of parameters gone. easy to share.
- [ ] MCP server
- [ ] Component from  selected Markdown files from `specs/`.
  - Use `include_str!` so docs are compiled into the app binary.
  - Render Markdown to HTML in a server function, not in the hydrate build.
  - Use `SsrMode::Async` for docs routes so crawlers receive rendered docs HTML.
  - Render the returned HTML inside the existing Leptos site shell so navbar/site interactivity
    remains available.
  - Start with a few flat pages: overview, frontend, tables, API, CLI/tooling, deployment.
  - Keep mdBook experiment as a reference, but do not deploy it unless the direction changes.

## Agent/tool access to insights

- [ ] Do not require agents or lightweight tools to download parquet files just to run curated
  insights. Keep local parquet execution for development, CI, and data verification, but make
  remote execution the default shape for agent integrations.
- [ ] Add REST endpoints backed by `exo_core::insights`:
  - `GET /rest/insights` — list slug/title/category/description/limit
  - `GET /rest/insights/{slug}` — run a validated predefined insight and return JSON rows
- [ ] Add optional remote mode to CLI after the REST endpoints exist:
  - `exo insights run <slug> --api-url https://exodata.space`
  - `exo insights list --api-url https://exodata.space`
- [ ] Build future MCP tools on top of the REST endpoint, not local parquet files:
  - `list_insights()`
  - `run_insight(slug)`
- [ ] Keep `exo_core::insights` as the canonical registry for what each insight means, but keep
  data access mode outside the core definition. Local CLI can execute SQL against parquet; web/API
  can execute against server-held dataframes and cache results.
- [ ] Consider static precomputed JSON snapshots for fixed top-N insights if runtime SQL becomes
  unnecessary or too expensive:
  - `/insights-data/index.json`
  - `/insights-data/{slug}.json`

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
