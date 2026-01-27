# Current Context

## Checklist before release

– [x] fix header layout on mobile
– [x] add counter from google analytics 
– [x] swagger & basic api with sql support
– [x] load of page should be overlay!
– [x] add proper about page
– [x] buy me a coffee button
– [x] add favicon
– [ ] link to the stellar system page (with planets if available)
- [ ] planet page
- [ ] when table pages fetch new data, "Select Columns" dissapearing, which looks strange

## REST API (current)

**Working endpoints**
- `GET /rest/stellarhosts` (pagination + sorting + columns)
- `GET /rest/exoplanets` (pagination + sorting + columns)
- `GET /rest/stellarhosts/schema`
- `GET /rest/exoplanets/schema`
- `GET /rest/query?sql=SELECT...&limit=...` (SELECT-only, max 10k rows, 30s timeout)
- `GET /rest/openapi.json`
- `GET /swagger-ui`

**Remaining TODO**
- Stats endpoints (`/rest/stats`, `/rest/stats/discoveries`, `/rest/stats/planets`)
- Export endpoints (`/rest/export/{table}`, `/rest/export/query`)
- Tests for new endpoints and error cases
- Middleware: CORS, rate limiting, request logging

## UI / Misc TODO

- Discovery timeline view (needs `disc_year` confirmation)
- Buymeacoffee button

