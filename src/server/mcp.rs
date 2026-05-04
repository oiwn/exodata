use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Implementation, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService,
        session::local::LocalSessionManager,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::ApiState;
use super::data::insights;

#[derive(Debug, Clone)]
pub struct ExodataMcp {
    state: ApiState,
}

impl ExodataMcp {
    fn new(state: ApiState) -> Self {
        Self { state }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct RunInsightRequest {
    /// Curated insight slug. Use list_insights to discover available slugs.
    slug: String,
}

pub fn mcp_routes(
    state: ApiState,
) -> StreamableHttpService<ExodataMcp, LocalSessionManager> {
    let config = StreamableHttpServerConfig::default()
        .with_allowed_hosts(allowed_hosts(&state))
        .with_stateful_mode(false)
        .with_json_response(true);
    let session_manager = Arc::new(LocalSessionManager::default());

    StreamableHttpService::new(
        move || Ok(ExodataMcp::new(state.clone())),
        session_manager,
        config,
    )
}

#[tool_router]
impl ExodataMcp {
    #[tool(description = "Check that the Exodata MCP server is alive.")]
    fn health(&self) -> CallToolResult {
        CallToolResult::structured(json!({
            "service": "exodata",
            "mcp": "ok",
            "version": env!("CARGO_PKG_VERSION"),
            "site_url": self.state.site_url.as_str(),
        }))
    }

    #[tool(description = "List curated Exodata insight tools and their slugs.")]
    fn list_insights(&self) -> CallToolResult {
        let insights = exo_core::insights::INSIGHTS
            .iter()
            .map(|def| {
                json!({
                    "slug": def.meta.slug,
                    "title": def.meta.title,
                    "category": def.meta.category,
                    "description": def.meta.description,
                    "kind": def.meta.kind,
                    "limit": def.meta.limit,
                })
            })
            .collect::<Vec<_>>();

        CallToolResult::structured(json!({ "insights": insights }))
    }

    #[tool(description = "Run a curated Exodata insight by slug.")]
    async fn run_insight(
        &self,
        Parameters(RunInsightRequest { slug }): Parameters<RunInsightRequest>,
    ) -> Result<CallToolResult, McpError> {
        let def = exo_core::insights::find_insight(&slug).ok_or_else(|| {
            McpError::invalid_params(
                format!("unknown insight slug: {slug}"),
                Some(json!({ "slug": slug })),
            )
        })?;

        let value = insights::get_insight_cached(
            &self.state.stellarhosts_df,
            &self.state.exoplanets_df,
            &self.state.insight_cache,
            &slug,
        )
        .await
        .map_err(|error| {
            McpError::internal_error(
                format!("failed to run insight {slug}: {error}"),
                Some(json!({ "slug": slug })),
            )
        })?;

        Ok(CallToolResult::structured(json!({
            "meta": {
                "slug": def.meta.slug,
                "title": def.meta.title,
                "category": def.meta.category,
                "description": def.meta.description,
                "kind": def.meta.kind,
                "limit": def.meta.limit,
            },
            "data": value.rows,
            "rows": value.total,
            "columns": value.columns,
        })))
    }
}

#[tool_handler]
impl ServerHandler for ExodataMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "exodata",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Read-only tools for Exoplanets Catalog curated insights.",
            )
    }
}

fn allowed_hosts(state: &ApiState) -> Vec<String> {
    let mut hosts = vec![
        "localhost".to_string(),
        "localhost:3000".to_string(),
        "127.0.0.1".to_string(),
        "127.0.0.1:3000".to_string(),
        "::1".to_string(),
    ];

    if let Some(host) = site_url_host(state.site_url.as_str()) {
        hosts.push(host);
    }

    hosts.sort();
    hosts.dedup();
    hosts
}

fn site_url_host(site_url: &str) -> Option<String> {
    let without_scheme = site_url
        .strip_prefix("https://")
        .or_else(|| site_url.strip_prefix("http://"))
        .unwrap_or(site_url);
    let host = without_scheme
        .split('/')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('/');

    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::site_url_host;

    #[test]
    fn site_url_host_extracts_authority() {
        assert_eq!(
            site_url_host("https://exodata.space/path"),
            Some("exodata.space".to_string())
        );
        assert_eq!(
            site_url_host("http://localhost:3000"),
            Some("localhost:3000".to_string())
        );
        assert_eq!(site_url_host(""), None);
    }
}
