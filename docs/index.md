# [Reason over precise exoplanet data](#mcp-exoplanet-data)

Exodata gives AI agents structured access to precise NASA Exoplanet Archive
data through a hosted MCP server.

Agents can inspect the catalog schema, compose SQL queries, and reason over the
returned records instead of relying on general model knowledge or manually
scanning large tables.

Because the agent discovers the available tables and columns at runtime, its
answers can stay grounded in catalog results. This is useful for comparisons,
filtered searches, rankings, and follow-up questions where exact values matter.

Connect a compatible MCP client to ask questions backed by the catalog's
current structured data.
