# MCP Server

Exoplanets Catalog exposes a read-only [Model Context Protocol](https://modelcontextprotocol.io)
endpoint at `/mcp` so coding agents and LLM tooling can query the catalog
directly.

Connection URLs:

- Hosted: `https://exodata.space/mcp`
- Local: `http://127.0.0.1:3000/mcp`

Transport: Streamable HTTP in stateless JSON response mode. Simple
request/response clients do not need to manage MCP session IDs.

## Tools

| Tool               | Description                                                              |
| ------------------ | ------------------------------------------------------------------------ |
| `health`           | Confirms the MCP server is alive.                                        |
| `list_insights`    | Returns curated insight metadata.                                        |
| `run_insight`      | Runs a curated insight by slug.                                          |
| `describe_catalog` | Returns column names, descriptions, units, and data types for a table.  |
| `query_catalog`    | Runs one read-only SQL `SELECT` against `stellarhosts` and `exoplanets`. |
| `download_detail`  | Returns one stellar host or exoplanet detail export as JSON or CSV.      |

`query_catalog` uses the same SQL validation and execution path as
[`/rest/query`](api.md#sql-query-endpoint).

## Limits

| Surface              | Default rows | Max rows |
| -------------------- | ------------ | -------- |
| MCP `query_catalog`  | 100          | 1000     |
| REST `/rest/query`   | 1000         | 10000    |

The MCP limits are stricter on purpose: agents pull rows into context windows,
so capping responses prevents one query from saturating the conversation.

Queries that fail validation, exceed the 30-second timeout, or target an
unregistered table return MCP errors with a descriptive message.

## Recommended Agent Flow

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

Example `download_detail` arguments:

```json
{
  "entity": "exoplanet",
  "name": "Kepler-22 b",
  "format": "json"
}
```

`download_detail` returns `filename`, `mime_type`, `content`, and the matching
browser download `url`. Supported entities are `stellarhost` and `exoplanet`;
supported formats are `json` and `csv`.

## Connecting an Agent

The hosted endpoint speaks Streamable HTTP, so any MCP client with native HTTP
transport support can connect without a local proxy. Each client uses a
slightly different config file name and key.

### Claude Code

Run from the project root, or `--scope user` for global:

```bash
claude mcp add --transport http exodata https://exodata.space/mcp
```

That writes `.mcp.json` (project scope) or `~/.claude.json` (user scope):

```json
{
  "mcpServers": {
    "exodata": {
      "type": "http",
      "url": "https://exodata.space/mcp"
    }
  }
}
```

### Crush

`.crush.json` in the project root or `~/.config/crush/crush.json`:

```json
{
  "$schema": "https://charm.land/crush.json",
  "mcp": {
    "exodata": {
      "type": "http",
      "url": "https://exodata.space/mcp"
    }
  }
}
```

### OpenCode

`opencode.json` in the project root or `~/.config/opencode/opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "exodata": {
      "type": "remote",
      "url": "https://exodata.space/mcp",
      "enabled": true
    }
  }
}
```

### Codex CLI

Codex speaks Streamable HTTP natively. The `codex mcp add` CLI only covers
stdio servers, so configure HTTP servers directly in `~/.codex/config.toml`:

```toml
[mcp_servers.exodata]
url = "https://exodata.space/mcp"
```

After configuration, the agent should see six tools: `health`, `list_insights`,
`run_insight`, `describe_catalog`, `query_catalog`, and `download_detail`. Call
`health` first to confirm the connection.
