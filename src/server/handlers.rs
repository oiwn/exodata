// REST API handlers for the exoplanets catalog
// These handlers use the shared business logic from common.rs

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use axum::{
    Router,
    extract::{OriginalUri, Path as AxumPath, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Json},
    routing::get,
};
use exo_core::metadata::ColumnMetadata;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use super::data::{insights, sql, tables};
use super::functions::DataStats;
use crate::server::cache::{HostDetailCache, InsightCache, TableCache};

#[derive(Debug, Clone)]
pub struct ApiState {
    pub site_url: Arc<String>,
    pub sitemap_index_xml: Arc<String>,
    pub sitemap_static_xml: Arc<String>,
    pub sitemap_entity_xml: Arc<HashMap<String, Arc<String>>>,
    pub stellarhosts_df: Arc<DataFrame>,
    pub exoplanets_df: Arc<DataFrame>,
    pub stellarhosts_metadata: Arc<HashMap<String, ColumnMetadata>>,
    pub exoplanets_metadata: Arc<HashMap<String, ColumnMetadata>>,
    pub overview_stats: Arc<DataStats>,
    pub table_cache: TableCache,
    pub host_detail_cache: HostDetailCache,
    pub insight_cache: InsightCache,
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
    /// Source datatype as declared in the column metadata file (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "double")]
    pub source_datatype: Option<String>,
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

#[derive(Debug, Serialize, ToSchema)]
pub struct InsightMetaResponse {
    pub slug: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub kind: String,
    pub limit: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InsightResponse {
    pub meta: InsightMetaResponse,
    #[schema(value_type = Vec<Object>)]
    pub data: Vec<Value>,
    pub rows: usize,
    pub columns: Vec<String>,
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
        get_insights,
        get_insight,
    ),
    components(
        schemas(ApiResponse, SchemaResponse, ColumnInfo, SqlResponse, InsightMetaResponse, InsightResponse)
    ),
    tags(
        (name = "data", description = "Data query endpoints"),
        (name = "schema", description = "Schema information endpoints"),
        (name = "sql", description = "SQL query endpoint"),
        (name = "insights", description = "Curated insight endpoints")
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
        .route("/insights", get(get_insights))
        .route("/insights/{slug}", get(get_insight))
        .with_state(state)
}

pub fn site_routes(state: ApiState) -> Router {
    let mut router = Router::new()
        .route("/sitemap-index.xml", get(get_sitemap_index))
        .route("/sitemap-static.xml", get(get_sitemap_static));

    for filename in state.sitemap_entity_xml.keys() {
        router = router.route(&format!("/{filename}"), get(get_sitemap_entity));
    }

    router.with_state(state)
}

/// Returns the Swagger UI router (should be mounted at root level, not nested)
pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/rest/openapi.json", ApiDoc::openapi())
}

async fn serve_xml(xml: Arc<String>) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml.as_str().to_owned(),
    )
}

pub async fn get_sitemap_index(
    State(state): State<ApiState>,
) -> impl IntoResponse {
    serve_xml(state.sitemap_index_xml).await
}

pub async fn get_sitemap_static(
    State(state): State<ApiState>,
) -> impl IntoResponse {
    serve_xml(state.sitemap_static_xml).await
}

pub async fn get_sitemap_entity(
    State(state): State<ApiState>,
    OriginalUri(uri): OriginalUri,
) -> impl IntoResponse {
    let filename = uri.path().trim_start_matches('/');
    match state.sitemap_entity_xml.get(filename) {
        Some(xml) => serve_xml(xml.clone()).await.into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
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
    let page = tables::normalize_table_page(params.page.unwrap_or(1));
    let limit = params.limit.unwrap_or(50).min(1000); // Cap at 1000

    // Parse columns parameter
    let selected_columns = params.columns.map(|s| {
        s.split(',')
            .map(|col| col.trim().to_string())
            .collect::<Vec<_>>()
    });

    // Use shared business logic from common.rs
    let value = tables::get_stellarhosts_data_cached(
        &state.stellarhosts_df,
        &state.table_cache,
        tables::TableQuery {
            page,
            limit,
            sort_by: params.sort_by,
            order: params.order,
            selected_columns,
            filter: params.filter,
        },
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get stellarhosts data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        data: value.rows,
        total: value.total,
        total_all: value.total_all,
        page,
        limit,
        columns: value.columns,
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
    let page = tables::normalize_table_page(params.page.unwrap_or(1));
    let limit = params.limit.unwrap_or(50).min(1000); // Cap at 1000

    // Parse columns parameter
    let selected_columns = params.columns.map(|s| {
        s.split(',')
            .map(|col| col.trim().to_string())
            .collect::<Vec<_>>()
    });

    // Use shared business logic from common.rs
    let value = tables::get_exoplanets_data_cached(
        &state.exoplanets_df,
        &state.table_cache,
        tables::TableQuery {
            page,
            limit,
            sort_by: params.sort_by,
            order: params.order,
            selected_columns,
            filter: params.filter,
        },
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get exoplanets data: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse {
        data: value.rows,
        total: value.total,
        total_all: value.total_all,
        page,
        limit,
        columns: value.columns,
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
    Json(build_schema_response(
        "stellarhosts",
        &state.stellarhosts_df,
        &state.stellarhosts_metadata,
    ))
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
    Json(build_schema_response(
        "exoplanets",
        &state.exoplanets_df,
        &state.exoplanets_metadata,
    ))
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
    let limit = params.limit.unwrap_or(1000).min(10000);
    let result = sql::execute_catalog_sql(
        state.stellarhosts_df.clone(),
        state.exoplanets_df.clone(),
        params.sql,
        limit,
    )
    .await
    .map_err(sql_error_status)?;

    Ok(Json(SqlResponse {
        data: result.data,
        rows: result.rows,
        columns: result.columns,
        query: result.query,
    }))
}

#[utoipa::path(
    get,
    path = "/rest/insights",
    responses(
        (status = 200, description = "Available curated insights", body = Vec<InsightMetaResponse>)
    ),
    tag = "insights"
)]
pub async fn get_insights() -> Json<Vec<InsightMetaResponse>> {
    Json(
        exo_core::insights::INSIGHTS
            .iter()
            .map(|def| insight_meta_response(def.meta))
            .collect(),
    )
}

#[utoipa::path(
    get,
    path = "/rest/insights/{slug}",
    params(
        ("slug" = String, Path, description = "Insight slug")
    ),
    responses(
        (status = 200, description = "Curated insight result", body = InsightResponse),
        (status = 404, description = "Unknown insight slug"),
        (status = 500, description = "Internal server error")
    ),
    tag = "insights"
)]
pub async fn get_insight(
    State(state): State<ApiState>,
    AxumPath(slug): AxumPath<String>,
) -> Result<Json<InsightResponse>, StatusCode> {
    let def =
        exo_core::insights::find_insight(&slug).ok_or(StatusCode::NOT_FOUND)?;
    let value = insights::get_insight_cached(
        &state.stellarhosts_df,
        &state.exoplanets_df,
        &state.insight_cache,
        &slug,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get insight {}: {}", slug, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(InsightResponse {
        meta: insight_meta_response(def.meta),
        data: value.rows,
        rows: value.total,
        columns: value.columns,
    }))
}

fn insight_meta_response(
    meta: &'static exo_types::insights::InsightMeta,
) -> InsightMetaResponse {
    InsightMetaResponse {
        slug: meta.slug.to_string(),
        title: meta.title.to_string(),
        category: meta.category.to_string(),
        description: meta.description.to_string(),
        kind: meta.kind.to_string(),
        limit: meta.limit,
    }
}

fn sql_error_status(error: sql::CatalogSqlError) -> StatusCode {
    match error {
        sql::CatalogSqlError::Timeout => {
            tracing::warn!("SQL query timed out");
            StatusCode::REQUEST_TIMEOUT
        }
        sql::CatalogSqlError::Join(error) => {
            tracing::error!("{error}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
        error => {
            tracing::warn!("SQL query failed: {error}");
            StatusCode::BAD_REQUEST
        }
    }
}

fn build_schema_response(
    table: &str,
    df: &DataFrame,
    metadata: &HashMap<String, ColumnMetadata>,
) -> SchemaResponse {
    let description = sql::describe_catalog(table, df, metadata, None)
        .expect("describe_catalog with None columns cannot fail");

    SchemaResponse {
        columns: description
            .columns
            .into_iter()
            .map(|column| ColumnInfo {
                name: column.name,
                data_type: column.data_type,
                description: column.description,
                unit: column.unit,
                source_datatype: column.source_datatype,
            })
            .collect(),
        total_rows: description.total_rows,
    }
}

const SITEMAP_CHUNK_SIZE: usize = 1_000;

const SITEMAP_PATH_SEGMENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'!')
    .add(b'"')
    .add(b'#')
    .add(b'$')
    .add(b'%')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

pub struct SitemapSet {
    pub index: String,
    pub static_pages: String,
    pub entity_sitemaps: BTreeMap<String, String>,
}

pub fn build_sitemaps(
    site_url: &str,
    build_date: &str,
    stellarhosts_df: &DataFrame,
    exoplanets_df: &DataFrame,
) -> Result<SitemapSet, String> {
    let site_url = site_url.trim_end_matches('/');

    let static_urls = build_static_urls(site_url);
    let stellarhost_urls = build_detail_urls(
        stellarhosts_df,
        "hostname",
        site_url,
        "/stellarhosts/",
    )?;
    let exoplanet_urls =
        build_detail_urls(exoplanets_df, "pl_name", site_url, "/exoplanets/")?;
    let stellarhost_sitemaps =
        build_chunked_sitemaps("stellarhosts", &stellarhost_urls, build_date);
    let exoplanet_sitemaps =
        build_chunked_sitemaps("exoplanets", &exoplanet_urls, build_date);

    let sitemap_filenames = std::iter::once("sitemap-static.xml".to_string())
        .chain(stellarhost_sitemaps.keys().cloned())
        .chain(exoplanet_sitemaps.keys().cloned())
        .collect::<Vec<_>>();
    let mut entity_sitemaps = BTreeMap::new();
    entity_sitemaps.extend(stellarhost_sitemaps);
    entity_sitemaps.extend(exoplanet_sitemaps);

    Ok(SitemapSet {
        index: render_sitemap_index(site_url, build_date, &sitemap_filenames),
        static_pages: render_urlset(&static_urls, build_date),
        entity_sitemaps,
    })
}

fn build_static_urls(site_url: &str) -> Vec<String> {
    let mut urls = vec![
        format!("{site_url}/"),
        format!("{site_url}/docs"),
        format!("{site_url}/docs/cli"),
        format!("{site_url}/docs/api"),
        format!("{site_url}/stellarhosts"),
        format!("{site_url}/exoplanets"),
        format!("{site_url}/insights"),
    ];
    urls.extend(
        exo_core::insights::INSIGHTS
            .iter()
            .map(|def| format!("{site_url}/insights/{}", def.meta.slug)),
    );
    urls
}

fn build_chunked_sitemaps(
    slug: &str,
    urls: &[String],
    build_date: &str,
) -> BTreeMap<String, String> {
    urls.chunks(SITEMAP_CHUNK_SIZE)
        .enumerate()
        .map(|(idx, chunk)| {
            (
                format!("sitemap-{slug}-{}.xml", idx + 1),
                render_urlset(chunk, build_date),
            )
        })
        .collect()
}

fn render_sitemap_index(
    site_url: &str,
    build_date: &str,
    sitemap_filenames: &[String],
) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<sitemapindex xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );

    for filename in sitemap_filenames {
        xml.push_str("  <sitemap>\n");
        xml.push_str(&format!("    <loc>{site_url}/{filename}</loc>\n"));
        if !build_date.is_empty() {
            xml.push_str(&format!("    <lastmod>{build_date}</lastmod>\n"));
        }
        xml.push_str("  </sitemap>\n");
    }

    xml.push_str("</sitemapindex>\n");
    xml
}

fn render_urlset(urls: &[String], build_date: &str) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );

    for url in urls {
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}</loc>\n", escape_xml(url)));
        if !build_date.is_empty() {
            xml.push_str(&format!("    <lastmod>{build_date}</lastmod>\n"));
        }
        xml.push_str("  </url>\n");
    }

    xml.push_str("</urlset>\n");
    xml
}

fn build_detail_urls(
    df: &DataFrame,
    column_name: &str,
    site_url: &str,
    route_prefix: &str,
) -> Result<Vec<String>, String> {
    let column = df
        .column(column_name)
        .map_err(|e| format!("Missing sitemap column '{column_name}': {e}"))?;
    let series = column.as_materialized_series();

    let mut unique_values = BTreeSet::new();
    for idx in 0..series.len() {
        if let Ok(value) = series.str_value(idx) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                unique_values.insert(trimmed.to_string());
            }
        }
    }

    Ok(unique_values
        .into_iter()
        .map(|value| {
            format!(
                "{site_url}{route_prefix}{}",
                utf8_percent_encode(&value, SITEMAP_PATH_SEGMENT_ENCODE_SET)
            )
        })
        .collect())
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
