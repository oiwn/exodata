# Current Context

## TODO

### Responsive Navigation Menu

**Problem**: On mobile (< 640px), the navigation menu is broken:
- Logo text "Exoplanets" overlaps with nav items
- Nav items get cut off (third item shows "Exo..." truncated)
- Items wrap awkwardly in two lines

**Solution**: Implement hamburger menu for mobile

**Requirements**:
- Desktop (>= 768px): Keep current horizontal nav layout
- Mobile (< 768px):
  - Show hamburger icon (3 horizontal lines) instead of nav items
  - Logo stays visible, can be smaller/icon-only
  - Tapping hamburger opens full-screen or slide-in menu overlay
  - Menu shows: Overview, Stellar Hosts, Exoplanets (vertical list)
  - Tap outside or X button closes menu

**Implementation approach**:
- Add mobile menu state (open/closed) with Leptos signal
- Use Tailwind responsive classes (`md:hidden`, `md:flex`)
- Hamburger button visible on mobile only
- Overlay/drawer component for mobile menu
- No JS framework needed - pure Leptos + Tailwind

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

### Swagger & SQL API

**Goal**: Provide REST API with SQL query support and OpenAPI documentation

**Requirements**:
- Swagger UI at `/api/docs` or `/swagger`
- Endpoints:
  - `GET /api/query?sql=...` - execute SQL query against parquet data
  - `GET /api/tables` - list available tables
  - `GET /api/schema/{table}` - get table schema
- Rate limiting and query validation (prevent destructive queries)
- Return JSON results with pagination

**Implementation**:
- Use `utoipa` crate for OpenAPI generation
- Use `datafusion` or polars SQL interface for queries
- Add to existing Axum `/rest` router
^^^ really? or let's split leptos server functions from REST api?

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
