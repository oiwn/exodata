# REST API

Exoplanets Catalog exposes NASA Exoplanet Archive data through JSON endpoints under `/rest`.
It also exposes a read-only hosted MCP endpoint at `/mcp` for agent clients.

Interactive OpenAPI documentation is available at <a href="/swagger-ui" rel="external">/swagger-ui</a>. The OpenAPI JSON document is available at <a href="/rest/openapi.json" rel="external">/rest/openapi.json</a>.

## Tables

The API exposes two catalog tables:

- `stellarhosts` - stellar host and system-level records
- `exoplanets` - confirmed exoplanet records with host/system fields

These are the same tables registered for SQL queries.

## Data Endpoints

### GET /rest/stellarhosts

Returns paginated stellar host rows.

Default columns:

```text
hostname, sy_dist, st_teff, st_mass, sy_pnum
```

### GET /rest/exoplanets

Returns paginated exoplanet rows.

Default columns:

```text
pl_name, hostname, discoverymethod, disc_year, pl_orbper, pl_rade, pl_bmasse
```

### Query Parameters

Both data endpoints accept the same parameters.

| Parameter | Type   | Default | Description                                      |
| --------- | ------ | ------- | ------------------------------------------------ |
| `page`    | number | `1`     | Page number, starting at 1.                      |
| `limit`   | number | `50`    | Rows per page. Values above 1000 are capped.     |
| `sort_by` | string |         | Column name to sort by.                          |
| `order`   | string | `asc`   | Sort direction: `asc` or `desc`.                 |
| `columns` | string |         | Comma-separated list of columns to return.       |
| `filter`  | string |         | Text filter applied by the table query pipeline. |

Example:

```bash
curl "https://exodata.space/rest/exoplanets?columns=pl_name,hostname,disc_year&sort_by=disc_year&order=desc&filter=Kepler&limit=20"
```

Response shape:

```json
{
  "data": [
    {
      "pl_name": "Kepler-452 b",
      "hostname": "Kepler-452",
      "disc_year": 2015
    }
  ],
  "total": 1523,
  "total_all": 5000,
  "page": 1,
  "limit": 20,
  "columns": ["pl_name", "hostname", "disc_year"]
}
```

`total` is the row count after filtering. `total_all` is the full table size before filtering.

## Schema Endpoints

### GET /rest/stellarhosts/schema

Returns column names, Polars data types, descriptions, units, and source
datatypes for the stellar host table.

### GET /rest/exoplanets/schema

Returns column names, Polars data types, descriptions, units, and source
datatypes for the exoplanet table.

Example:

```bash
curl "https://exodata.space/rest/exoplanets/schema"
```

Response shape:

```json
{
  "columns": [
    {
      "name": "pl_name",
      "data_type": "String",
      "description": "Planet Name",
      "source_datatype": "char"
    },
    {
      "name": "pl_orbper",
      "data_type": "Float64",
      "description": "Orbital Period",
      "unit": "days",
      "source_datatype": "double"
    }
  ],
  "total_rows": 5000
}
```

`description`, `unit`, and `source_datatype` are omitted when metadata is
unavailable for a column. `data_type` is the runtime Polars dtype;
`source_datatype` is the type declared in the column metadata TOML.

## SQL Query Endpoint

### GET /rest/query

Executes one read-only SQL `SELECT` statement against the registered catalog tables.

Available tables:

- `stellarhosts`
- `exoplanets`

Query parameters:

| Parameter | Type   | Default | Description                                  |
| --------- | ------ | ------- | -------------------------------------------- |
| `sql`     | string |         | Required SQL `SELECT` statement.             |
| `limit`   | number | `1000`  | Maximum rows returned. Capped at 10000 rows. |

Only one SQL statement is allowed. Non-`SELECT` statements are rejected. Queries run with a 30-second timeout.

Examples:

```bash
curl "https://exodata.space/rest/query?sql=SELECT pl_name, hostname, disc_year FROM exoplanets ORDER BY disc_year DESC LIMIT 10"
```

```bash
curl "https://exodata.space/rest/query?sql=SELECT s.hostname, s.st_teff, e.pl_name FROM stellarhosts s JOIN exoplanets e ON s.hostname = e.hostname LIMIT 10"
```

```bash
curl "https://exodata.space/rest/query?sql=SELECT discoverymethod, COUNT(*) AS count FROM exoplanets GROUP BY discoverymethod ORDER BY count DESC"
```

Response shape:

```json
{
  "data": [
    {
      "pl_name": "Kepler-452 b",
      "hostname": "Kepler-452",
      "disc_year": 2015
    }
  ],
  "rows": 1,
  "columns": ["pl_name", "hostname", "disc_year"],
  "query": "SELECT pl_name, hostname, disc_year FROM exoplanets ORDER BY disc_year DESC LIMIT 10"
}
```

## Insight Endpoints

### GET /rest/insights

Returns available curated insights.

Example:

```bash
curl "https://exodata.space/rest/insights"
```

Response shape:

```json
[
  {
    "slug": "nearest-stellar-hosts",
    "title": "Nearest Stellar Hosts",
    "category": "stellarhosts",
    "description": "Nearest stellar hosts in the catalog",
    "kind": "ranking",
    "limit": 10
  }
]
```

### GET /rest/insights/{slug}

Runs one predefined curated insight and returns JSON rows.

Example:

```bash
curl "https://exodata.space/rest/insights/nearest-stellar-hosts"
```

Response shape:

```json
{
  "meta": {
    "slug": "nearest-stellar-hosts",
    "title": "Nearest Stellar Hosts",
    "category": "stellarhosts",
    "description": "Nearest stellar hosts in the catalog",
    "kind": "ranking",
    "limit": 10
  },
  "data": [
    {
      "hostname": "Proxima Cen",
      "sy_dist": 1.30119
    }
  ],
  "rows": 1,
  "columns": ["hostname", "sy_dist"]
}
```

## MCP Endpoint

### /mcp

Hosted Model Context Protocol endpoint using Streamable HTTP.

Connection URLs:

- Local: `http://127.0.0.1:3000/mcp`
- Hosted: `https://exodata.space/mcp`

Tools:

- `health` - confirms the MCP server is alive.
- `list_insights` - returns curated insight metadata.
- `run_insight` - runs a curated insight by slug.
- `describe_catalog` - returns column names, descriptions, units, and data
  types for `stellarhosts` or `exoplanets`.
- `query_catalog` - runs one read-only SQL `SELECT` query against
  `stellarhosts` and `exoplanets`.

The MCP surface is read-only. `query_catalog` uses the same SQL validation and
execution path as `/rest/query`, but defaults to 100 returned rows and caps MCP
responses at 1000 rows. The server runs in stateless JSON response mode, so
simple request/response clients do not need to manage MCP session IDs.

Recommended agent flow:

1. Call `describe_catalog` for the table and columns relevant to the task.
2. Use the returned metadata to write a SQL `SELECT`.
3. Call `query_catalog` with the SQL and optional `limit`.

Example `describe_catalog` arguments:

```json
{
  "table": "exoplanets",
  "columns": ["pl_name", "hostname", "pl_rade", "pl_eqt"]
}
```

Example `query_catalog` arguments:

```json
{
  "sql": "SELECT pl_name, hostname, pl_rade FROM exoplanets WHERE pl_rade BETWEEN 0.8 AND 1.2 ORDER BY pl_rade",
  "limit": 25
}
```

Join example:

```json
{
  "sql": "SELECT s.hostname, s.st_teff, e.pl_name, e.pl_rade FROM stellarhosts s JOIN exoplanets e ON s.hostname = e.hostname ORDER BY e.pl_rade LIMIT 10"
}
```

Aggregate example:

```json
{
  "sql": "SELECT discoverymethod, COUNT(*) AS count FROM exoplanets GROUP BY discoverymethod ORDER BY count DESC",
  "limit": 20
}
```

## Errors

The API uses standard HTTP status codes:

| Status | Meaning                                    |
| ------ | ------------------------------------------ |
| `400`  | Invalid query parameters or invalid SQL.   |
| `404`  | Unknown insight slug.                      |
| `408`  | SQL query timed out.                       |
| `500`  | Server-side data loading or query failure. |
