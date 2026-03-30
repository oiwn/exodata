# REST API

Base URL: `/rest`

Interactive docs available at `/swagger-ui` (OpenAPI spec at `/rest/openapi.json`).

## Endpoints

### GET /rest/stellarhosts

Paginated stellar hosts data with sorting, column selection, and text filtering.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | usize | 1 | Page number |
| `limit` | usize | 50 | Rows per page (max: 1000) |
| `sort_by` | string | — | Column name to sort by |
| `order` | string | `asc` | Sort order: `asc` or `desc` |
| `columns` | string | — | Comma-separated list of columns to return |
| `filter` | string | — | Text filter (partial match across all visible columns) |

**Example:**

```bash
curl "http://localhost:3000/rest/stellarhosts?page=1&limit=20&sort_by=st_teff&order=desc&filter=Kepler"
```

**Response:**

```json
{
  "data": [
    { "hostname": "Kepler-452", "st_teff": 5757.0, "st_mass": 1.04, ... }
  ],
  "total": 1523,
  "total_all": 5200,
  "page": 1,
  "limit": 20,
  "columns": ["hostname", "st_teff", "st_mass"]
}
```

### GET /rest/exoplanets

Same parameters as `/rest/stellarhosts`, but for the exoplanets dataset.

```bash
curl "http://localhost:3000/rest/exoplanets?filter=Kepler&sort_by=disc_year&order=desc"
```

### GET /rest/stellarhosts/schema

Column metadata for stellar hosts.

**Response:**

```json
{
  "columns": [
    {
      "name": "hostname",
      "type": "String",
      "description": "Stellar host name",
      "unit": ""
    }
  ],
  "total_rows": 5200
}
```

### GET /rest/exoplanets/schema

Column metadata for exoplanets. Same structure as `/rest/stellarhosts/schema`.

### GET /rest/query

Execute SQL SELECT queries against both tables.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `sql` | string | required | SQL SELECT statement |
| `limit` | usize | — | Override result limit (max: 10,000) |

Available tables: `stellarhosts`, `exoplanets`

**Examples:**

```bash
# Find hot Jupiters
curl "http://localhost:3000/rest/query?sql=SELECT pl_name, pl_bmasse, pl_eqt FROM exoplanets WHERE pl_bmasse > 100 AND pl_eqt > 1000 LIMIT 10"

# Join tables
curl "http://localhost:3000/rest/query?sql=SELECT s.hostname, s.st_teff, e.pl_name FROM stellarhosts s JOIN exoplanets e ON s.hostname = e.hostname LIMIT 10"

# Count by discovery method
curl "http://localhost:3000/rest/query?sql=SELECT discoverymethod, COUNT(*) as cnt FROM exoplanets GROUP BY discoverymethod ORDER BY cnt DESC"
```

Only `SELECT` statements are allowed. Queries run with a 30-second timeout.
