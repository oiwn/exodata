use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::tables::common;

#[derive(Debug, Clone)]
pub struct ApiState {
    pub stellarhosts_df: Arc<DataFrame>,
    pub exoplanets_df: Arc<DataFrame>,
}

/// Parameters for filtering, sorting, and pagination
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryParams {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub sort_by: Option<String>,
    pub order: Option<String>, // asc or desc
    // Generic filter parameters
    pub hostname: Option<String>,
    pub pl_name: Option<String>,
    // Numeric range filters
    pub sy_dist_min: Option<f64>,
    pub sy_dist_max: Option<f64>,
    pub st_teff_min: Option<f64>,
    pub st_teff_max: Option<f64>,
    // Exoplanet specific filters
    pub pl_orbper_min: Option<f64>,
    pub pl_orbper_max: Option<f64>,
    pub pl_rade_min: Option<f64>,
    pub pl_rade_max: Option<f64>,
    pub pl_masse_min: Option<f64>,
    pub pl_masse_max: Option<f64>,
}

/// Response structure for paginated data
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub limit: usize,
    pub filters: QueryParams,
}

pub fn api_routes() -> Router {
    // Load dataframes at startup
    let stellarhosts_df = match common::load_parquet("data/stellarhosts.parquet", None) {
        Ok(df) => Arc::new(df),
        Err(e) => panic!("Failed to load stellarhosts data: {}", e),
    };

    let exoplanets_df = match common::load_parquet("data/exoplanets.parquet", None) {
        Ok(df) => Arc::new(df),
        Err(e) => panic!("Failed to load exoplanets data: {}", e),
    };

    let state = ApiState {
        stellarhosts_df,
        exoplanets_df,
    };

    Router::new()
        .route("/stellarhosts", get(get_stellarhosts))
        .route("/exoplanets", get(get_exoplanets))
        .route("/stellarhosts/schema", get(get_stellarhosts_schema))
        .route("/exoplanets/schema", get(get_exoplanets_schema))
        .with_state(state)
}

/// Handler for stellarhosts data with filtering and pagination
pub async fn get_stellarhosts(
    State(state): State<ApiState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let mut df = (*state.stellarhosts_df).clone();

    // Apply filters
    df = apply_stellarhosts_filters(df, &params)?;

    // Get total count before pagination
    let total = df.height();

    // Apply sorting
    if let Some(sort_by) = &params.sort_by {
        df = apply_sorting(df, sort_by, params.order.as_deref())?;
    }

    // Apply pagination
    df = apply_pagination(df, params.page, params.limit)?;

    // Convert to JSON
    let data = dataframe_to_json(&df)?;

    let response = ApiResponse {
        data,
        total,
        page: params.page.unwrap_or(1),
        limit: params.limit.unwrap_or(50),
        filters: params,
    };

    Ok(Json(response))
}

/// Handler for exoplanets data with filtering and pagination
pub async fn get_exoplanets(
    State(state): State<ApiState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<ApiResponse<Value>>, StatusCode> {
    let mut df = (*state.exoplanets_df).clone();

    // Apply filters
    df = apply_exoplanets_filters(df, &params)?;

    // Get total count before pagination
    let total = df.height();

    // Apply sorting
    if let Some(sort_by) = &params.sort_by {
        df = apply_sorting(df, sort_by, params.order.as_deref())?;
    }

    // Apply pagination
    df = apply_pagination(df, params.page, params.limit)?;

    // Convert to JSON
    let data = dataframe_to_json(&df)?;

    let response = ApiResponse {
        data,
        total,
        page: params.page.unwrap_or(1),
        limit: params.limit.unwrap_or(50),
        filters: params,
    };

    Ok(Json(response))
}

/// Handler for stellarhosts schema information
pub async fn get_stellarhosts_schema(
    State(state): State<ApiState>,
) -> Json<Value> {
    let df = &*state.stellarhosts_df;
    let schema = dataframe_schema_to_json(df);
    Json(schema)
}

/// Handler for exoplanets schema information
pub async fn get_exoplanets_schema(
    State(state): State<ApiState>,
) -> Json<Value> {
    let df = &*state.exoplanets_df;
    let schema = dataframe_schema_to_json(df);
    Json(schema)
}

/// Apply filters specific to stellarhosts dataset
fn apply_stellarhosts_filters(
    df: DataFrame,
    params: &QueryParams,
) -> Result<DataFrame, StatusCode> {
    let mut df = df;

    // Text filters - simple string contains
    if let Some(hostname) = &params.hostname {
        if let Ok(col) = df.column("hostname") {
            if let Ok(str_col) = col.str() {
                // Create a boolean mask for filtering
                let mut mask = BooleanChunkedBuilder::new(col.name().clone(), str_col.len());
                for i in 0..str_col.len() {
                    let contains = str_col.get(i)
                        .map(|s| s.to_lowercase().contains(&hostname.to_lowercase()))
                        .unwrap_or(false);
                    mask.append_value(contains);
                }
                let filter_mask = mask.finish();
                df = df.filter(&filter_mask).unwrap_or(df);
            }
        }
    }

    // Numeric range filters
    if let Some(min_val) = params.sy_dist_min {
        if let Ok(col) = df.column("sy_dist") {
            if let Ok(f64_col) = col.f64() {
                let mask = f64_col.gt_eq(min_val);
                df = df.filter(&mask).unwrap_or(df);
            }
        }
    }

    if let Some(max_val) = params.sy_dist_max {
        if let Ok(col) = df.column("sy_dist") {
            if let Ok(f64_col) = col.f64() {
                let mask = f64_col.lt_eq(max_val);
                df = df.filter(&mask).unwrap_or(df);
            }
        }
    }

    if let Some(min_val) = params.st_teff_min {
        if let Ok(col) = df.column("st_teff") {
            if let Ok(f64_col) = col.f64() {
                let mask = f64_col.gt_eq(min_val);
                df = df.filter(&mask).unwrap_or(df);
            }
        }
    }

    if let Some(max_val) = params.st_teff_max {
        if let Ok(col) = df.column("st_teff") {
            if let Ok(f64_col) = col.f64() {
                let mask = f64_col.lt_eq(max_val);
                df = df.filter(&mask).unwrap_or(df);
            }
        }
    }

    Ok(df)
}

/// Apply filters specific to exoplanets dataset
fn apply_exoplanets_filters(
    df: DataFrame,
    params: &QueryParams,
) -> Result<DataFrame, StatusCode> {
    let mut df = df;

    // Text filters
    if let Some(hostname) = &params.hostname {
        if let Ok(col) = df.column("hostname") {
            if let Ok(str_col) = col.str() {
                // Create a boolean mask for filtering
                let mut mask = BooleanChunkedBuilder::new(col.name().clone(), str_col.len());
                for i in 0..str_col.len() {
                    let contains = str_col.get(i)
                        .map(|s| s.to_lowercase().contains(&hostname.to_lowercase()))
                        .unwrap_or(false);
                    mask.append_value(contains);
                }
                let filter_mask = mask.finish();
                df = df.filter(&filter_mask).unwrap_or(df);
            }
        }
    }

    if let Some(pl_name) = &params.pl_name {
        if let Ok(col) = df.column("pl_name") {
            if let Ok(str_col) = col.str() {
                // Create a boolean mask for filtering
                let mut mask = BooleanChunkedBuilder::new(col.name().clone(), str_col.len());
                for i in 0..str_col.len() {
                    let contains = str_col.get(i)
                        .map(|s| s.to_lowercase().contains(&pl_name.to_lowercase()))
                        .unwrap_or(false);
                    mask.append_value(contains);
                }
                let filter_mask = mask.finish();
                df = df.filter(&filter_mask).unwrap_or(df);
            }
        }
    }

    // Numeric range filters
    if let Some(min_val) = params.pl_orbper_min {
        if let Ok(col) = df.column("pl_orbper") {
            if let Ok(f64_col) = col.f64() {
                let mask = f64_col.gt_eq(min_val);
                df = df.filter(&mask).unwrap_or(df);
            }
        }
    }

    if let Some(max_val) = params.pl_orbper_max {
        if let Ok(col) = df.column("pl_orbper") {
            if let Ok(f64_col) = col.f64() {
                let mask = f64_col.lt_eq(max_val);
                df = df.filter(&mask).unwrap_or(df);
            }
        }
    }

    if let Some(min_val) = params.pl_rade_min {
        if let Ok(col) = df.column("pl_rade") {
            if let Ok(f64_col) = col.f64() {
                let mask = f64_col.gt_eq(min_val);
                df = df.filter(&mask).unwrap_or(df);
            }
        }
    }

    if let Some(max_val) = params.pl_rade_max {
        if let Ok(col) = df.column("pl_rade") {
            if let Ok(f64_col) = col.f64() {
                let mask = f64_col.lt_eq(max_val);
                df = df.filter(&mask).unwrap_or(df);
            }
        }
    }

    if let Some(min_val) = params.pl_masse_min {
        if let Ok(col) = df.column("pl_masse") {
            if let Ok(f64_col) = col.f64() {
                let mask = f64_col.gt_eq(min_val);
                df = df.filter(&mask).unwrap_or(df);
            }
        }
    }

    if let Some(max_val) = params.pl_masse_max {
        if let Ok(col) = df.column("pl_masse") {
            if let Ok(f64_col) = col.f64() {
                let mask = f64_col.lt_eq(max_val);
                df = df.filter(&mask).unwrap_or(df);
            }
        }
    }

    Ok(df)
}

/// Apply sorting to a dataframe
fn apply_sorting(
    df: DataFrame,
    sort_by: &str,
    order: Option<&str>,
) -> Result<DataFrame, StatusCode> {
    let descending = order.unwrap_or("asc") == "desc";

    let options = SortMultipleOptions::new()
        .with_order_descending(descending);
    
    match df.sort([sort_by], options) {
        Ok(sorted_df) => Ok(sorted_df),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

/// Apply pagination to a dataframe
fn apply_pagination(
    df: DataFrame,
    page: Option<usize>,
    limit: Option<usize>,
) -> Result<DataFrame, StatusCode> {
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(50);
    
    if page == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let offset = (page - 1) * limit;
    if offset >= df.height() {
        return Ok(DataFrame::empty());
    }

    let end = std::cmp::min(offset + limit, df.height());
    
    Ok(df.slice(offset as i64, end - offset))
}

/// Convert a DataFrame to a JSON value
fn dataframe_to_json(df: &DataFrame) -> Result<Vec<Value>, StatusCode> {
    let mut rows = Vec::new();
    let columns = df.get_column_names();
    
    for row_idx in 0..df.height() {
        let mut row_map = HashMap::new();
        
        for col_name in &columns {
            if let Ok(col) = df.column(col_name) {
                let value = match col.dtype() {
                    DataType::String => {
                        col.str().unwrap()
                            .get(row_idx)
                            .map(|s| json!(s))
                            .unwrap_or(json!(null))
                    }
                    DataType::Float64 => {
                        col.f64().unwrap()
                            .get(row_idx)
                            .map(|f| json!(f))
                            .unwrap_or(json!(null))
                    }
                    DataType::Int64 => {
                        col.i64().unwrap()
                            .get(row_idx)
                            .map(|i| json!(i))
                            .unwrap_or(json!(null))
                    }
                    _ => json!(null),
                };
                row_map.insert(col_name.to_string(), value);
            }
        }
        
        rows.push(json!(row_map));
    }
    
    Ok(rows)
}

/// Convert DataFrame schema to JSON for UI column selection
fn dataframe_schema_to_json(df: &DataFrame) -> Value {
    let mut columns = Vec::new();
    
    for field in df.fields() {
        let column_info = json!({
            "name": field.name(),
            "data_type": format!("{:?}", field.dtype()),
        });
        columns.push(column_info);
    }
    
    json!({
        "columns": columns,
        "total_rows": df.height()
    })
}