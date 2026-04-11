# Current Context

## Stellar Hosts Table Refactor

### Focus

- Refactor `src/components/stellarhosts_table.rs` into smaller, easier-to-scan pieces.
- Use this refactor to establish a cleaner styling pattern for this page.

### Current Assessment

- `StellarHostsTablePage` currently mixes too many responsibilities:
  - query param parsing
  - reactive table state
  - resource loading and loading-overlay state
  - URL synchronization/navigation
  - page chrome/header/meta
  - pagination controls
  - table rendering
  - loading and error UI
- Navigation/query rebuilding logic is repeated across sort, filter, prev/next, and page-link handlers.
- Pagination markup is duplicated at the top and bottom of the page.
- Large inline Tailwind class strings reduce readability of the Rust view code.

### Styling Decision

- Choose semantic classes with feature CSS files.
- Do not move toward one large shared stylesheet as the main pattern.
- Do not introduce scoped styling tooling in this pass.
- For this page, add a dedicated stylesheet such as `style/components/stellarhosts-table.scss`.
- Import that stylesheet into `style/main.scss`.
- Keep only small, obvious utility classes inline when they help readability.
- Move repeated or section-level styling into semantic classes owned by the feature stylesheet.

### Refactor Direction

- Split page shell/chrome from data-table controller logic where practical.
- Extract reusable pagination controls into a smaller component.
- Extract loading and error states into smaller view helpers or components.
- Centralize URL/query-string navigation logic in one helper path instead of repeating it in every handler.
- Preserve current behavior and current visual design while improving readability and maintainability.

## Implementation Phases

### Phase 1: Query State Extraction

- Status: completed
- Introduce shared table query state and shared navigation helper in `src/table/`.
- Move query-building consumers onto the shared state model.
- Add focused tests for the shared query/navigation layer.

### Phase 2: Table Page Structure Split

- Status: pending
- Break `src/components/stellarhosts_table.rs` into smaller internal pieces.
- Separate page chrome/meta from table controller logic.
- Extract loading, error, and pagination view sections into smaller components or helpers.

### Phase 3: Repeated Interaction Cleanup

- Status: pending
- Remove duplicated handler logic for sort, filter, and page transitions.
- Centralize state-to-URL synchronization paths.
- Reduce repeated pagination control markup.

### Phase 4: Styling Refactor

- Status: pending
- Create `style/components/stellarhosts-table.scss`.
- Move repeated section-level Tailwind class piles into semantic classes.
- Import the feature stylesheet from `style/main.scss`.
- Preserve the current visual design while improving readability.

### Phase 5: Validation

- Status: pending
- Run formatting and compile/tests.
- Manually verify table interactions still behave the same.
- Clean up `specs/ctx.md` when the temporary task notes are no longer needed.
