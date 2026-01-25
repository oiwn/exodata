# Current Context

## Checklist before release

– [x] fix header layout on mobile
– [ ] add counter from google analytics 
– [x] swagger & basic api with sql support
– [ ] if it possible to show diff between dates (planet discovery date field available)
– [ ] load of page should be overlay!
– [ ] add proper about page
– [ ] buy me a coffee button
– [x] add favicon
– [ ]  link to the stellar system page (with planets if available)

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
- Full-page loading overlay (replace current inline indicator)
- Google Analytics (env-based GA4)
- Buymeacoffee button

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
