# Connect to Exodata's exoplanet MCP server

Exodata provides hosted [Model Context Protocol](https://modelcontextprotocol.io)
access to NASA Exoplanet Archive records. Connect an AI agent to inspect catalog
schemas, run read-only SQL, explore curated insights, and export a planet or
stellar host as JSON or CSV. Exodata is an independent service, not operated by NASA.

Connection URL:

- Hosted: `https://exodata.space/mcp`

Transport: Streamable HTTP in stateless JSON response mode. Simple
request/response clients do not need to manage MCP session IDs.

## Quick setup for Codex, Claude Code, and OpenCode

Choose your client; no local MCP server or proxy installation is needed.

Codex CLI:

```bash
codex mcp add exodata --url https://exodata.space/mcp
```

Claude Code, shared project configuration:

```bash
claude mcp add --scope project --transport http exodata https://exodata.space/mcp
```

OpenCode, project configuration:

```bash
opencode mcp add exodata --url https://exodata.space/mcp
```

These commands save client configuration. Restart or reconnect your client as
needed, then ask it to call Exodata's `health` tool to check the connection.
The server exposes the six tools listed below. Configuration details and other
HTTP clients are covered under Connecting an Agent.

## Supported data and refresh behavior

### Which NASA Exoplanet Archive tables can I query?

Exodata loads prepared exports of NASA's `stellarhosts` and `ps` tables. Query
them as `stellarhosts` and `exoplanets` respectively. Inspect available fields,
descriptions, units, and types with `describe_catalog`; see the
[NASA PS column definitions](https://exoplanetarchive.ipac.caltech.edu/docs/API_PS_columns.html)
for source field documentation.

### Does each row represent a different planet?

No. `exoplanets` contains PS reference records: multiple rows can describe the
same planet using different references. `COUNT(*)` counts reference records;
use `COUNT(DISTINCT pl_name)` to count distinct planet names. Host joins can
also return multiple matching records. Check the schema and choose the rows
appropriate for your question before interpreting a comparison or ranking.

### Is the data queried live from NASA?

No. Queries run against the prepared catalog loaded at Exodata server startup,
not against NASA's live service. New exports require conversion and a server
restart before they appear. Exodata does not promise a daily refresh schedule.
Record fields such as `rowupdate` describe source rows, not the service's last
refresh date.

### Can I use SQL and download results?

Use `query_catalog` for a single read-only SQL `SELECT`, including supported
joins and aggregations. This is Exodata's SQL interface, not an upstream NASA
ADQL endpoint. Query responses are bounded by the limits below; `download_detail`
exports one named planet or host as JSON or CSV, rather than an unlimited query
result. Browse the [catalog](https://exodata.space/exoplanets),
[stellar hosts](https://exodata.space/stellarhosts), or [REST API guide](api.md)
for related access paths.

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

Ask for reference records with planet radii between 0.8 and 1.2 Earth radii.
Inspect the returned records rather than treating this radius filter as a
habitability assessment; one planet may occur more than once.

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

This counts reference records by discovery method, not distinct planets.

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

Run from the project root with explicit project scope:

```bash
claude mcp add --scope project --transport http exodata https://exodata.space/mcp
```

That writes `.mcp.json`. Replace `--scope project` with `--scope user` for
global configuration in `~/.claude.json`. Omitting `--scope` defaults to a
private local-project entry in `~/.claude.json`, not `.mcp.json`.
See [Claude Code's MCP guide](https://code.claude.com/docs/en/mcp).

Project configuration:

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

Add this to `crushrc` in the project root or `~/.config/crush/crushrc`,
following the [Crush configuration guide](https://github.com/charmbracelet/crush/tree/main/docs/config):

```bash
mcp add exodata --type http --url https://exodata.space/mcp
```

Existing JSON configurations (`.crush.json`, `crush.json`, or
`~/.config/crush/crush.json`) remain supported as a legacy format:

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

The quick-setup command writes to project configuration. Add `--global` to
write to global configuration instead. Alternatively, add a remote entry to
`opencode.json` in the project root or
`~/.config/opencode/opencode.json`, following the
[OpenCode MCP guide](https://opencode.ai/docs/mcp-servers/):

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

Codex supports Streamable HTTP natively. The quick-setup command above adds
the hosted URL to `~/.codex/config.toml`. You can instead add the entry directly,
following the [official Codex MCP guide](https://developers.openai.com/codex/mcp):

```toml
[mcp_servers.exodata]
url = "https://exodata.space/mcp"
```

After configuration, the agent should see six tools: `health`, `list_insights`,
`run_insight`, `describe_catalog`, `query_catalog`, and `download_detail`. Call
`health` first to confirm the connection.
