# Current Context

## TODO

- [x] responsive navigation menu (should looks good on mobile)

---

### Google Analytics Integration

**Goal**: Track page views and user interactions

**Requirements**:
- Add GA4 tracking script to page head
- Track page views for: Overview, Stellar Hosts, Exoplanets
- Store GA measurement ID in environment variable or config
- Respect user privacy (consider cookie consent if needed)

**Implementation**:
- Add GA script in `shell()` function or layout component
- Use `LEPTOS_GA_ID` env var or hardcode measurement ID

---

### Swagger & REST API

**Goal**: Provide comprehensive REST API with SQL query support, statistics, export formats, and OpenAPI documentation

#### URL Prefix Convention

- **`/api/*`** - Reserved for Leptos server functions (auto-registered by `#[server]` macro)
- **`/rest/*`** - Public REST API
- **`/swagger-ui`** - Swagger UI (mounted at root level)

#### Current Endpoints (working)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/rest/stellarhosts` | Paginated stellar hosts with sorting & column selection |
| GET | `/rest/exoplanets` | Paginated exoplanets with sorting & column selection |
| GET | `/rest/stellarhosts/schema` | Column metadata with descriptions & units |
| GET | `/rest/exoplanets/schema` | Column metadata with descriptions & units |
| GET | `/rest/openapi.json` | OpenAPI 3.1 specification |
| GET | `/swagger-ui` | Interactive API documentation |

#### Implementation Steps

- [x] **Step 1: Refactor handlers.rs to use common.rs**
  - Remove duplicate filtering/pagination/sorting logic (lines 154-367)
  - Call `common::get_stellarhosts_data()` and `common::get_exoplanets_data()`
  - Simplify `QueryParams` to generic: `page`, `limit`, `sort_by`, `order`, `columns`
  - Remove hardcoded column filters (`hostname`, `sy_dist_min`, etc.)

- [x] **Step 2: Add utoipa annotations**
  - Add `utoipa` and `utoipa-swagger-ui` to Cargo.toml
  - Annotate `QueryParams`, `ApiResponse` with `#[derive(ToSchema)]`
  - Annotate handlers with `#[utoipa::path(...)]`
  - Create `ApiDoc` struct with `#[derive(OpenApi)]`

- [x] **Step 3: Mount Swagger UI**
  - Swagger UI at `/swagger-ui` (root level, not nested under /rest)
  - OpenAPI JSON at `/rest/openapi.json`
  - Added "API" link to navbar (desktop & mobile)

- [x] **Step 4: Add metadata endpoints** (already done via schema endpoints)
  - `/rest/stellarhosts/schema` - column metadata with types, descriptions, units
  - `/rest/exoplanets/schema` - same for exoplanets
  - `/rest/tables` not needed (only 2 tables)

- [ ] **Step 5: Add statistics endpoints**
  - `GET /rest/stats` - reuse logic from `get_stats()` in functions.rs
  - `GET /rest/stats/discoveries` - group by year or method
  - Extract shared logic to `common.rs` if needed

- [ ] **Step 6: Add export endpoints**
  - `GET /rest/export/{table}` - export with format param (csv, json)
  - Implement CSV serialization (use `polars` or manual)
  - Add `columns` param for column selection
  - Add `limit` param (default 1000, max 10000)

- [ ] **Step 7: Add SQL query endpoint**
  - `GET /rest/query?sql=...` - execute SQL against parquet
  - Use `polars` SQLContext to parse and execute
  - Validate query: only SELECT allowed
  - Add timeout (30s) and row limit (10000)

- [ ] **Step 8: Update tests**
  - Update `src/server/tests.rs` to match new API
  - Add tests for new endpoints (stats, export, query)
  - Test error cases (invalid SQL, missing params)

- [ ] **Step 9: Add middleware**
  - CORS headers for external consumers
  - Rate limiting (optional, use `tower-governor`)
  - Request logging

#### Planned Endpoints (at `/rest`)

**Data Endpoints**
```
GET /rest/stellarhosts?columns=...&page=1&limit=50&sort_by=...&order=asc
GET /rest/exoplanets?columns=...&page=1&limit=50&sort_by=...&order=asc
```

**SQL Query Endpoint**
```
GET /rest/query?sql=SELECT...&limit=100
```
- Execute read-only SQL against parquet data
- Use `polars` SQLContext for SQL parsing
- Whitelist: SELECT only
- Block: DROP, DELETE, UPDATE, INSERT, CREATE, ALTER
- Max result limit: 10,000 rows
- Timeout: 30 seconds

**Statistics Endpoints**
```
GET /rest/stats
GET /rest/stats/discoveries?group_by=year
GET /rest/stats/discoveries?group_by=method
GET /rest/stats/planets?group_by=size_category
```

**Export Endpoints**
```
GET /rest/export/stellarhosts?format=csv&columns=hostname,sy_dist,st_teff
GET /rest/export/exoplanets?format=json&limit=1000
GET /rest/export/query?sql=SELECT...&format=csv
```

Formats: `csv`, `json`, `parquet`

**Metadata Endpoints**
```
GET /rest/tables
GET /rest/columns/{table}
```

#### OpenAPI/Swagger (done)

- Swagger UI at `/swagger-ui`
- OpenAPI JSON at `/rest/openapi.json`
- Using `utoipa` v5.4 + `utoipa-swagger-ui` v9.0
- "API" link in navbar opens Swagger UI

#### Implementation Notes

- `src/server/handlers.rs` - refactored, uses `common.rs`
- `ApiState` shared with Leptos functions
- Tests updated in `src/server/tests.rs`
- Still TODO: CORS, rate limiting, additional endpoints

---

### Discovery Timeline / Diff View

**Goal**: Visualize exoplanet discoveries over time
^^^ No, need to check if discovery date available.

**Requirements**:
- Use `disc_year` (discovery year) field from exoplanets data
- Show:
  - Timeline chart of discoveries per year
  - Filter/compare between date ranges
  - "New discoveries since [date]" view
- Could be a new page or section on Overview

**Implementation**:
- Server function to aggregate discoveries by year
- Frontend chart component (consider lightweight charting lib or pure CSS/SVG)
- Date range picker for comparisons

---

### Page Loading Overlay

**Goal**: Show loading indicator while initial data loads
^^^ there is indicator already, but i would like to make it on top of the page, so content will change only at the last moment

**Problem**: Page may appear blank or broken while SSR hydrates / data loads

**Requirements**:
- Full-screen overlay with spinner/animation on initial load
- Overlay disappears when hydration complete and data ready
- Should not flash on fast connections (delay before showing)
- Branded loading state (logo + "Loading..." text)

**Implementation**:
- CSS-only initial loader in HTML (no JS dependency)
- Leptos `Suspense` or `Transition` for data loading states
- Remove overlay on `on_mount` or when resources resolve

### Integrate Buymeacoffe button

```html
  <a href="https://www.buymeacoffee.com/oiwn"><img src="https://img.buymeacoffee.com/button-api/?text=Buy me a coffee&emoji=&slug=oiwn&button_colour=BD5FFF&font_colour=ffffff&font_family=Lato&outline_colour=000000&coffee_colour=FFDD00" /></a>
```

### Google Analytics code:

```html
<!-- Google tag (gtag.js) -->
<script async src="https://www.googletagmanager.com/gtag/js?id=G-MHKPES88ZJ"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());

  gtag('config', 'G-MHKPES88ZJ');
</script>
``` 
