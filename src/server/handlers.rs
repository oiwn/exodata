// REST API handlers for the exoplanets catalog
// These handlers use the shared business logic from common.rs

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use exo_core::metadata::ColumnMetadata;
use polars::prelude::*;
use polars::sql::SQLContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use utoipa::{IntoParams, OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use super::common;
use super::functions::DataStats;
use crate::server::cache::TableCache;

#[derive(Debug, Clone)]
pub struct ApiState {
    pub stellarhosts_df: Arc<DataFrame>,
    pub exoplanets_df: Arc<DataFrame>,
    pub stellarhosts_metadata: Arc<HashMap<String, ColumnMetadata>>,
    pub exoplanets_metadata: Arc<HashMap<String, ColumnMetadata>>,
    pub overview_stats: Arc<DataStats>,
    pub table_cache: TableCache,
}

/// Generic query parameters for data endpoints
/// Column-agnostic design - works with any table
#[derive(Debug, Deserialize, Serialize, IntoParams)]
pub struct QueryParams {
    /// Page number (1-indexed, default: 1)
    #[param(example = 1)]
    pub page: Option<usize>,
    /// Number of rows per page (default: 50, max: 1000)
    #[param(example = 50)]
    pub limit: Option<usize>,
    /// Column name to sort by
    #[param(example = "hostname")]
    pub sort_by: Option<String>,
    /// Sort order: "asc" or "desc" (default: "asc")
    #[param(example = "asc")]
    pub order: Option<String>,
    /// Comma-separated list of columns to return
    #[param(example = "hostname,sy_dist,st_teff")]
    pub columns: Option<String>,
    /// Text filter applied to the first selected column
    #[param(example = "Kepler")]
    pub filter: Option<String>,
}

/// Response structure for paginated data
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse {
    /// Array of row objects
    #[schema(value_type = Vec<Object>)]
    pub data: Vec<Value>,
    /// Total rows matching query (after filtering, before pagination)
    #[schema(example = 5000)]
    pub total: usize,
    /// Total rows in entire dataset (unfiltered)
    #[schema(example = 5000)]
    pub total_all: usize,
    /// Current page number
    #[schema(example = 1)]
    pub page: usize,
    /// Rows per page
    #[schema(example = 50)]
    pub limit: usize,
    /// Column names in the response
    #[schema(example = json!(["hostname", "sy_dist", "st_teff"]))]
    pub columns: Vec<String>,
}

/// Response structure for schema endpoint
#[derive(Debug, Serialize, ToSchema)]
pub struct SchemaResponse {
    /// List of columns with their metadata
    pub columns: Vec<ColumnInfo>,
    /// Total rows in the table
    #[schema(example = 5000)]
    pub total_rows: usize,
}

/// Column metadata information
#[derive(Debug, Serialize, ToSchema)]
pub struct ColumnInfo {
    /// Column name
    #[schema(example = "hostname")]
    pub name: String,
    /// Data type (e.g., "String", "Float64", "Int64")
    #[schema(example = "String")]
    pub data_type: String,
    /// Human-readable description of the column
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Host star name")]
    pub description: Option<String>,
    /// Unit of measurement (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "parsec")]
    pub unit: Option<String>,
}

/// Query parameters for SQL endpoint
#[derive(Debug, Deserialize, Serialize, IntoParams)]
pub struct SqlQueryParams {
    /// SQL query to execute (SELECT only)
    #[param(
        example = "SELECT pl_name, hostname, disc_year FROM exoplanets LIMIT 10"
    )]
    pub sql: String,
    /// Maximum number of rows to return (default: 1000, max: 10000)
    #[param(example = 1000)]
    pub limit: Option<usize>,
}

/// Response structure for SQL query
#[derive(Debug, Serialize, ToSchema)]
pub struct SqlResponse {
    /// Array of row objects
    #[schema(value_type = Vec<Object>)]
    pub data: Vec<Value>,
    /// Number of rows returned
    #[schema(example = 100)]
    pub rows: usize,
    /// Column names in the result
    #[schema(example = json!(["pl_name", "hostname", "disc_year"]))]
    pub columns: Vec<String>,
    /// The SQL query that was executed
    pub query: String,
}

/// OpenAPI documentation for the Exoplanets Catalog REST API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Exoplanets Catalog API",
        version = "1.0.0",
        description = "REST API for querying the NASA Exoplanet Archive data. Provides access to stellar hosts and exoplanets with pagination, sorting, and column selection.",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT"),
    ),
    paths(
        get_stellarhosts,
        get_exoplanets,
        get_stellarhosts_schema,
        get_exoplanets_schema,
        execute_sql,
    ),
    components(
        schemas(ApiResponse, SchemaResponse, ColumnInfo, SqlResponse)
    ),
    tags(
        (name = "data", description = "Data query endpoints"),
        (name = "schema", description = "Schema information endpoints"),
        (name = "sql", description = "SQL query endpoint")
    )
)]
pub struct ApiDoc;

pub fn api_routes(state: ApiState) -> Router {
    Router::new()
        .route("/stellarhosts", get(get_stellarhosts))
        .route("/exoplanets", get(get_exoplanets))
        .route("/stellarhosts/schema", get(get_stellarhosts_schema))
        .route("/exoplanets/schema", get(get_exoplanets_schema))
        .route("/query", get(execute_sql))
        .with_state(state)
}

/// Returns the Swagger UI router (should be mounted at root level, not nested)
pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/rest/openapi.json", ApiDoc::openapi())
}

/// Get stellar hosts data
///
/// Returns paginated stellar host data with optional sorting and column selection.
/// Default columns: hostname, sy_dist, st_teff, st_mass, sy_pnum
#[utoipa::path(
    get,
    path = "/rest/stellarhosts",
    params(QueryParams),
    responses(
        (status = 200, description = "Stellar hosts data", body = ApiResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "data"
)]
pub async fn get_stellarhosts(
    State(state): State<ApiState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(50).min(1000); // Cap at 1000

    // Parse columns parameter
    let selected_columns = params.columns.map(|s| {
        s.split(',')
            .map(|col| col.trim().to_string())
            .collect::<Vec<_>>()
    });

    // Use shared business logic from common.rs
    let (rows, total, total_all, columns) =
        common::get_stellarhosts_data_cached(
            &state.stellarhosts_df,
            &state.stellarhosts_metadata,
            &state.table_cache,
            page,
            limit,
            params.sort_by,
            params.order,
            selected_columns,
            params.filter,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to get stellarhosts data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        data: rows,
        total,
        total_all,
        page,
        limit,
        columns,
    }))
}

/// Get exoplanets data
///
/// Returns paginated exoplanet data with optional sorting and column selection.
/// Default columns: pl_name, hostname, discoverymethod, disc_year, pl_orbper, pl_rade, pl_bmasse
#[utoipa::path(
    get,
    path = "/rest/exoplanets",
    params(QueryParams),
    responses(
        (status = 200, description = "Exoplanets data", body = ApiResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "data"
)]
pub async fn get_exoplanets(
    State(state): State<ApiState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(50).min(1000); // Cap at 1000

    // Parse columns parameter
    let selected_columns = params.columns.map(|s| {
        s.split(',')
            .map(|col| col.trim().to_string())
            .collect::<Vec<_>>()
    });

    // Use shared business logic from common.rs
    let (rows, total, total_all, columns) =
        common::get_exoplanets_data_cached(
            &state.exoplanets_df,
            &state.exoplanets_metadata,
            &state.table_cache,
            page,
            limit,
            params.sort_by,
            params.order,
            selected_columns,
            params.filter,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to get exoplanets data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ApiResponse {
        data: rows,
        total,
        total_all,
        page,
        limit,
        columns,
    }))
}

/// Get stellar hosts schema
///
/// Returns column metadata for the stellar hosts table including column names,
/// data types, descriptions, and units.
#[utoipa::path(
    get,
    path = "/rest/stellarhosts/schema",
    responses(
        (status = 200, description = "Stellar hosts schema", body = SchemaResponse)
    ),
    tag = "schema"
)]
pub async fn get_stellarhosts_schema(
    State(state): State<ApiState>,
) -> Json<SchemaResponse> {
    let df = &*state.stellarhosts_df;
    let metadata = &*state.stellarhosts_metadata;
    let schema = build_schema_response(df, metadata);
    Json(schema)
}

/// Get exoplanets schema
///
/// Returns column metadata for the exoplanets table including column names,
/// data types, descriptions, and units.
#[utoipa::path(
    get,
    path = "/rest/exoplanets/schema",
    responses(
        (status = 200, description = "Exoplanets schema", body = SchemaResponse)
    ),
    tag = "schema"
)]
pub async fn get_exoplanets_schema(
    State(state): State<ApiState>,
) -> Json<SchemaResponse> {
    let df = &*state.exoplanets_df;
    let metadata = &*state.exoplanets_metadata;
    let schema = build_schema_response(df, metadata);
    Json(schema)
}

/// Execute a SQL query against the registered tables.
///
/// Tables: stellarhosts, exoplanets
#[utoipa::path(
    get,
    path = "/rest/query",
    params(SqlQueryParams),
    responses(
        (status = 200, description = "SQL query result", body = SqlResponse),
        (status = 400, description = "Invalid SQL query"),
        (status = 408, description = "SQL query timed out"),
        (status = 500, description = "Internal server error")
    ),
    tag = "sql"
)]
pub async fn execute_sql(
    State(state): State<ApiState>,
    Query(params): Query<SqlQueryParams>,
) -> Result<Json<SqlResponse>, StatusCode> {
    let query = params.sql.trim().to_string();
    if query.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    validate_sql_select_only(&query)?;

    let limit = params.limit.unwrap_or(1000).min(10000);
    let query_for_exec = query.clone();
    let stellarhosts_df = state.stellarhosts_df.clone();
    let exoplanets_df = state.exoplanets_df.clone();

    let handle = tokio::task::spawn_blocking(move || {
        let mut ctx = SQLContext::new();
        ctx.register("stellarhosts", stellarhosts_df.as_ref().clone().lazy());
        ctx.register("exoplanets", exoplanets_df.as_ref().clone().lazy());

        let lazy = ctx
            .execute(&query_for_exec)
            .map_err(|e| format!("SQL execution error: {}", e))?;
        let df = lazy
            .limit(limit as IdxSize)
            .collect()
            .map_err(|e| format!("Failed to collect SQL result: {}", e))?;

        let rows = df.height();
        let columns = df
            .get_column_names()
            .iter()
            .map(|name| (*name).to_string())
            .collect::<Vec<_>>();
        let data = common::dataframe_to_json(&df)?;

        Ok::<_, String>((data, columns, rows))
    });

    let result = tokio::time::timeout(Duration::from_secs(30), handle).await;
    match result {
        Err(_) => {
            tracing::warn!("SQL query timed out");
            Err(StatusCode::REQUEST_TIMEOUT)
        }
        Ok(join_result) => match join_result {
            Err(err) => {
                tracing::error!("SQL query join error: {}", err);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
            Ok(Err(err)) => {
                tracing::warn!("SQL query failed: {}", err);
                Err(StatusCode::BAD_REQUEST)
            }
            Ok(Ok((data, columns, rows))) => Ok(Json(SqlResponse {
                data,
                rows,
                columns,
                query,
            })),
        },
    }
}

fn validate_sql_select_only(query: &str) -> Result<(), StatusCode> {
    let dialect = GenericDialect {};
    let statements = Parser::parse_sql(&dialect, query)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if statements.len() != 1 {
        return Err(StatusCode::BAD_REQUEST);
    }

    match statements.first() {
        Some(sqlparser::ast::Statement::Query(_)) => Ok(()),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

/// Build schema response with column metadata
fn build_schema_response(
    df: &DataFrame,
    metadata: &HashMap<String, ColumnMetadata>,
) -> SchemaResponse {
    let columns: Vec<ColumnInfo> = df
        .fields()
        .iter()
        .map(|field| {
            let name = field.name().to_string();
            let meta = metadata.get(&name);
            ColumnInfo {
                name: name.clone(),
                data_type: format!("{:?}", field.dtype()),
                description: meta.and_then(|m| m.description.clone()),
                unit: meta.and_then(|m| m.unit.clone()),
            }
        })
        .collect();

    SchemaResponse {
        columns,
        total_rows: df.height(),
    }
}
