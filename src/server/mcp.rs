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
use super::data::{exports, insights, sql};

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

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct QueryCatalogRequest {
    /// SQL SELECT query to run against the stellarhosts and exoplanets tables.
    sql: String,
    /// Maximum rows to return. Defaults to 100 and is capped at 1000.
    limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct DescribeCatalogRequest {
    /// Catalog table to describe: "stellarhosts" or "exoplanets".
    table: String,
    /// Optional subset of column names to describe.
    columns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct DownloadDetailRequest {
    /// Entity type: "stellarhost" or "exoplanet".
    entity: String,
    /// Detail page entity name, such as a hostname or planet name.
    name: String,
    /// Export format: "json" or "csv".
    format: String,
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

    #[tool(
        description = "Describe Exodata catalog columns for a table before writing SQL. Tables: stellarhosts, exoplanets."
    )]
    fn describe_catalog(
        &self,
        Parameters(request): Parameters<DescribeCatalogRequest>,
    ) -> Result<CallToolResult, McpError> {
        let columns = request.columns.as_deref();
        let description = match request.table.as_str() {
            "stellarhosts" => sql::describe_catalog(
                "stellarhosts",
                &self.state.stellarhosts_df,
                &self.state.stellarhosts_metadata,
                columns,
            ),
            "exoplanets" => sql::describe_catalog(
                "exoplanets",
                &self.state.exoplanets_df,
                &self.state.exoplanets_metadata,
                columns,
            ),
            table => {
                Err(sql::CatalogSchemaError::UnknownTable(table.to_string()))
            }
        }
        .map_err(schema_error)?;

        Ok(CallToolResult::structured(json!(description)))
    }

    #[tool(
        description = "Run one read-only SQL SELECT query against the Exodata catalog. Tables: stellarhosts, exoplanets. Default limit is 100; maximum limit is 1000."
    )]
    async fn query_catalog(
        &self,
        Parameters(request): Parameters<QueryCatalogRequest>,
    ) -> Result<CallToolResult, McpError> {
        let limit = request.limit.unwrap_or(100).min(1000);
        let result = sql::execute_catalog_sql(
            self.state.stellarhosts_df.clone(),
            self.state.exoplanets_df.clone(),
            request.sql,
            limit,
        )
        .await
        .map_err(sql_error)?;

        Ok(CallToolResult::structured(json!(result)))
    }

    #[tool(
        description = "Download one stellar host or exoplanet detail export. Entities: stellarhost, exoplanet. Formats: json, csv."
    )]
    async fn download_detail(
        &self,
        Parameters(request): Parameters<DownloadDetailRequest>,
    ) -> Result<CallToolResult, McpError> {
        let entity = exports::ExportEntity::parse(&request.entity)
            .map_err(export_param_error)?;
        let format = exports::ExportFormat::parse(&request.format)
            .map_err(export_param_error)?;

        let export = match entity {
            exports::ExportEntity::StellarHost => {
                exports::export_stellarhost(
                    &self.state.stellarhosts_df,
                    &self.state.host_detail_cache,
                    &self.state.stellarhosts_metadata,
                    self.state.site_url.as_str(),
                    &request.name,
                    format,
                )
                .await
            }
            exports::ExportEntity::Exoplanet => exports::export_exoplanet(
                &self.state.exoplanets_df,
                &self.state.exoplanets_metadata,
                self.state.site_url.as_str(),
                &request.name,
                format,
            ),
        }
        .map_err(export_data_error)?;

        Ok(CallToolResult::structured(json!({
            "filename": export.filename,
            "mime_type": export.mime_type,
            "content": export.content,
            "url": export.url,
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
                "Read-only tools for Exoplanets Catalog. Use describe_catalog to inspect table columns before calling query_catalog with SQL SELECT statements.",
            )
    }
}

fn schema_error(error: sql::CatalogSchemaError) -> McpError {
    McpError::invalid_params(
        error.to_string(),
        Some(json!({ "error": error.to_string() })),
    )
}

fn sql_error(error: sql::CatalogSqlError) -> McpError {
    match error {
        sql::CatalogSqlError::Timeout | sql::CatalogSqlError::Join(_) => {
            McpError::internal_error(
                error.to_string(),
                Some(json!({ "error": error.to_string() })),
            )
        }
        _ => McpError::invalid_params(
            error.to_string(),
            Some(json!({ "error": error.to_string() })),
        ),
    }
}

fn export_param_error(error: String) -> McpError {
    McpError::invalid_params(error.clone(), Some(json!({ "error": error })))
}

fn export_data_error(error: String) -> McpError {
    if error.contains("not found") {
        McpError::invalid_params(error.clone(), Some(json!({ "error": error })))
    } else {
        McpError::internal_error(error.clone(), Some(json!({ "error": error })))
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
    use std::collections::HashMap;
    use std::sync::Arc;

    use exo_types::metadata::ColumnMetadata;
    use polars::prelude::*;

    use super::*;
    use crate::server::cache::{
        build_host_detail_cache, build_insight_cache, build_table_cache,
    };
    use crate::server::functions::DataStats;

    fn create_test_state() -> ApiState {
        let stellarhosts_df = df! {
            "hostname" => &["HD 189733", "Kepler-22", "HD 209458"],
            "sy_dist" => &[19.3, 600.0, 47.9],
            "st_teff" => &[5040.0, 5518.0, 6092.0],
        }
        .unwrap();

        let exoplanets_df = df! {
            "pl_name" => &["HD 189733 b", "Kepler-22 b", "HD 209458 b"],
            "hostname" => &["HD 189733", "Kepler-22", "HD 209458"],
            "discoverymethod" => &["Radial Velocity", "Transit", "Transit"],
            "disc_year" => &[2005i64, 2011i64, 1999i64],
            "pl_rade" => &[1.138, 2.38, 1.32],
            "pl_eqt" => &[1201.0, 262.0, 1450.0],
        }
        .unwrap();

        let mut exoplanets_metadata = HashMap::new();
        exoplanets_metadata.insert(
            "pl_rade".to_string(),
            ColumnMetadata {
                name: "pl_rade".to_string(),
                description: Some("Planet Radius".to_string()),
                unit: Some("Rearth".to_string()),
                datatype: "double".to_string(),
            },
        );
        exoplanets_metadata.insert(
            "pl_eqt".to_string(),
            ColumnMetadata {
                name: "pl_eqt".to_string(),
                description: Some("Equilibrium Temperature".to_string()),
                unit: Some("K".to_string()),
                datatype: "double".to_string(),
            },
        );

        ApiState {
            site_url: Arc::new("https://example.com".to_string()),
            sitemap_index_xml: Arc::new(String::new()),
            sitemap_static_xml: Arc::new(String::new()),
            sitemap_entity_xml: Arc::new(HashMap::new()),
            stellarhosts_df: Arc::new(stellarhosts_df),
            exoplanets_df: Arc::new(exoplanets_df),
            stellarhosts_metadata: Arc::new(HashMap::new()),
            exoplanets_metadata: Arc::new(exoplanets_metadata),
            overview_stats: Arc::new(DataStats {
                stellarhosts_total: 0,
                exoplanets_total: 0,
                avg_stellar_temp: 0.0,
                avg_stellar_distance: 0.0,
                discovery_methods: Vec::new(),
                planet_size_categories: Vec::new(),
                planet_temperature_bands: Vec::new(),
                detection_sources: Vec::new(),
                discovery_years: Vec::new(),
                orbital_period_buckets: Vec::new(),
            }),
            table_cache: build_table_cache(64),
            host_detail_cache: build_host_detail_cache(64),
            insight_cache: build_insight_cache(16),
        }
    }

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

    #[test]
    fn mcp_tool_router_lists_agent_catalog_tools() {
        let tools = ExodataMcp::tool_router().list_all();
        let tool_names = tools
            .iter()
            .map(|tool| tool.name.as_ref())
            .collect::<Vec<_>>();

        assert!(tool_names.contains(&"health"));
        assert!(tool_names.contains(&"list_insights"));
        assert!(tool_names.contains(&"run_insight"));
        assert!(tool_names.contains(&"describe_catalog"));
        assert!(tool_names.contains(&"query_catalog"));
        assert!(tool_names.contains(&"download_detail"));
    }

    #[test]
    fn mcp_initialize_info_advertises_tools_and_agent_instructions() {
        let mcp = ExodataMcp::new(create_test_state());
        let info = mcp.get_info();

        assert_eq!(info.server_info.name, "exodata");
        assert!(info.capabilities.tools.is_some());
        assert!(
            info.instructions
                .as_deref()
                .unwrap()
                .contains("describe_catalog")
        );
    }

    #[test]
    fn describe_catalog_returns_requested_column_metadata() {
        let mcp = ExodataMcp::new(create_test_state());

        let result = mcp
            .describe_catalog(Parameters(DescribeCatalogRequest {
                table: "exoplanets".to_string(),
                columns: Some(vec!["pl_rade".to_string(), "pl_eqt".to_string()]),
            }))
            .unwrap();
        let content = result.structured_content.unwrap();

        assert_eq!(content["table"], "exoplanets");
        assert_eq!(content["columns"].as_array().unwrap().len(), 2);
        assert_eq!(content["columns"][0]["name"], "pl_rade");
        assert_eq!(content["columns"][0]["unit"], "Rearth");
        assert_eq!(content["columns"][1]["name"], "pl_eqt");
    }

    #[tokio::test]
    async fn query_catalog_runs_select_query() {
        let mcp = ExodataMcp::new(create_test_state());

        let result = mcp
            .query_catalog(Parameters(QueryCatalogRequest {
                sql: "SELECT pl_name, hostname FROM exoplanets ORDER BY disc_year DESC"
                    .to_string(),
                limit: Some(2),
            }))
            .await
            .unwrap();
        let content = result.structured_content.unwrap();

        assert_eq!(content["rows"], 2);
        assert_eq!(content["columns"], json!(["pl_name", "hostname"]));
        assert_eq!(content["data"][0]["pl_name"], "Kepler-22 b");
    }

    #[tokio::test]
    async fn query_catalog_caps_limit_at_1000() {
        let mcp = ExodataMcp::new(create_test_state());

        let result = mcp
            .query_catalog(Parameters(QueryCatalogRequest {
                sql: "SELECT pl_name FROM exoplanets".to_string(),
                limit: Some(10_000),
            }))
            .await
            .unwrap();
        let content = result.structured_content.unwrap();

        assert_eq!(content["rows"], 3);
    }

    #[tokio::test]
    async fn query_catalog_rejects_invalid_sql() {
        let mcp = ExodataMcp::new(create_test_state());

        let error = mcp
            .query_catalog(Parameters(QueryCatalogRequest {
                sql: "SELECT FROM".to_string(),
                limit: None,
            }))
            .await
            .unwrap_err();

        assert!(error.message.contains("invalid SQL syntax"));
    }

    #[tokio::test]
    async fn query_catalog_rejects_non_select_sql() {
        let mcp = ExodataMcp::new(create_test_state());

        let error = mcp
            .query_catalog(Parameters(QueryCatalogRequest {
                sql: "DELETE FROM exoplanets".to_string(),
                limit: None,
            }))
            .await
            .unwrap_err();

        assert!(error.message.contains("only SELECT"));
    }

    #[tokio::test]
    async fn query_catalog_rejects_multiple_statements() {
        let mcp = ExodataMcp::new(create_test_state());

        let error = mcp
            .query_catalog(Parameters(QueryCatalogRequest {
                sql: "SELECT * FROM exoplanets; SELECT * FROM stellarhosts"
                    .to_string(),
                limit: None,
            }))
            .await
            .unwrap_err();

        assert!(error.message.contains("exactly one statement"));
    }

    #[tokio::test]
    async fn query_catalog_returns_execution_error_for_unregistered_table() {
        let mcp = ExodataMcp::new(create_test_state());

        let error = mcp
            .query_catalog(Parameters(QueryCatalogRequest {
                sql: "SELECT * FROM missing_table".to_string(),
                limit: None,
            }))
            .await
            .unwrap_err();

        assert!(error.message.contains("SQL execution error"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn download_detail_returns_exoplanet_json_export() {
        let mcp = ExodataMcp::new(create_test_state());

        let result = mcp
            .download_detail(Parameters(DownloadDetailRequest {
                entity: "exoplanet".to_string(),
                name: "Kepler-22 b".to_string(),
                format: "json".to_string(),
            }))
            .await
            .unwrap();
        let content = result.structured_content.unwrap();

        assert_eq!(content["filename"], "exoplanet-Kepler-22_b.json");
        assert_eq!(content["mime_type"], "application/json; charset=utf-8");
        assert_eq!(
            content["url"],
            "https://example.com/exoplanets/Kepler%2D22%20b.json"
        );
        assert!(content["content"].as_str().unwrap().contains("Kepler-22 b"));
    }

    #[tokio::test]
    async fn download_detail_rejects_invalid_format() {
        let mcp = ExodataMcp::new(create_test_state());

        let error = mcp
            .download_detail(Parameters(DownloadDetailRequest {
                entity: "exoplanet".to_string(),
                name: "Kepler-22 b".to_string(),
                format: "toon".to_string(),
            }))
            .await
            .unwrap_err();

        assert!(error.message.contains("format must be 'json' or 'csv'"));
    }
}
