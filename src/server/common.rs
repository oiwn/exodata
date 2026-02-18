//! Common business logic for server operations
//! This module contains pure functions that are called by both
//! Leptos server functions and Axum REST handlers.
//! It's server-only and contains no HTTP/Leptos dependencies.

use super::cache::{
    TableCache, TableCacheValue, TableKind, normalize_table_cache_key,
};
use exo_core::metadata::ColumnMetadata;
use polars::prelude::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

/// Result type for stellarhosts data operations
/// Returns: (rows, filtered_total, unfiltered_total, columns)
pub type TableResult = Result<(Vec<Value>, usize, usize, Vec<String>), String>;

/// Result type for stellarhosts data operations
/// Returns: (rows, filtered_total, unfiltered_total, columns)
pub type StellarHostsResult = TableResult;

/// Result type for exoplanets data operations
/// Returns: (rows, filtered_total, unfiltered_total, columns)
pub type ExoplanetsResult = TableResult;

/// Table configuration for generic data queries.
pub struct TableConfig<'a> {
    pub default_columns: &'a [&'a str],
}

/// Generic table data query with shared pagination/sort/select logic.
pub fn get_table_data(
    df: &DataFrame,
    _all_metadata: &Arc<HashMap<String, ColumnMetadata>>,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    selected_columns: Option<Vec<String>>,
    filter: Option<String>,
    config: TableConfig<'_>,
) -> TableResult {
    // Clone the dataframe to work with it
    let mut df = df.clone();

    // Use selected columns or fall back to defaults
    let columns_to_select: Vec<&str> = if let Some(cols) = &selected_columns {
        // Validate that requested columns exist in dataframe
        cols.iter()
            .filter(|col| df.column(col).is_ok())
            .map(|s| s.as_str())
            .collect()
    } else {
        config.default_columns.iter().copied().collect()
    };

    // Ensure we have at least one column
    if columns_to_select.is_empty() {
        return Err("No valid columns selected".to_string());
    }

    // Select only the columns we need
    df = df
        .select(columns_to_select.clone())
        .map_err(|e| format!("Failed to select columns: {}", e))?;

    // Get unfiltered total FIRST (before any filtering)
    let total_all = df.height();

    // Apply first-column text filter if provided
    if let Some(filter_value) = filter {
        let needle = filter_value.trim().to_lowercase();
        if !needle.is_empty() {
            if let Some(first_col) = columns_to_select.first() {
                let series = df.column(first_col).map_err(|e| {
                    format!("Failed to read filter column: {}", e)
                })?;
                let series = if matches!(series.dtype(), DataType::String) {
                    series.clone()
                } else {
                    series.cast(&DataType::String).map_err(|e| {
                        format!("Failed to cast filter column: {}", e)
                    })?
                };
                let utf8 = series.str().map_err(|e| {
                    format!("Failed to read string column: {}", e)
                })?;
                let mask: BooleanChunked = utf8
                    .into_iter()
                    .map(|opt| opt.map(|s| s.to_lowercase().contains(&needle)))
                    .collect();
                df = df
                    .filter(&mask)
                    .map_err(|e| format!("Failed to apply filter: {}", e))?;
            }
        }
    }

    // Apply sorting with null filtering if requested
    // Only sort if the sort column is actually in the selected columns
    if let Some(sort_col) = &sort_by {
        if columns_to_select.contains(&sort_col.as_str()) {
            // Filter out rows where sort column is null using LazyFrame
            df = df
                .lazy()
                .filter(col(sort_col).is_not_null())
                .collect()
                .map_err(|e| format!("Failed to filter nulls: {}", e))?;

            // Then sort
            let descending = order.as_deref().unwrap_or("asc") == "desc";
            let options =
                SortMultipleOptions::new().with_order_descending(descending);

            df = df
                .sort([sort_col.as_str()], options)
                .map_err(|e| format!("Failed to sort: {}", e))?;
        }
        // If sort column is not in selected columns, silently ignore the sort
    }

    // Get filtered total AFTER filtering (but before pagination)
    let total = df.height();

    // Apply pagination
    let page = if page == 0 { 1 } else { page };
    let offset = (page - 1) * limit;

    if offset < df.height() {
        let end = std::cmp::min(offset + limit, df.height());
        df = df.slice(offset as i64, end - offset);
    } else {
        // Return empty dataframe if offset is beyond data
        df = df.slice(0, 0);
    }

    // Convert DataFrame to JSON
    let rows = dataframe_to_json(&df)?;

    // Get column names
    let columns: Vec<String> =
        columns_to_select.iter().map(|s| (*s).to_string()).collect();

    Ok((rows, total, total_all, columns))
}

/// Get paginated stellar hosts data with sorting
///
/// This is the core business logic that both server functions and REST handlers use.
/// It performs Polars operations and returns the data in a simple format.
///
/// # Arguments
/// * `df` - Reference to the stellarhosts DataFrame
/// * `all_metadata` - Reference to all column metadata
/// * `page` - Page number (1-indexed)
/// * `limit` - Number of rows per page
/// * `sort_by` - Optional column name to sort by
/// * `order` - Sort order ("asc" or "desc")
/// * `selected_columns` - Optional list of column names to display
///
/// # Returns
/// A tuple of (rows as JSON, filtered_total, unfiltered_total, column names)
pub fn get_stellarhosts_data(
    df: &DataFrame,
    all_metadata: &Arc<HashMap<String, ColumnMetadata>>,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    selected_columns: Option<Vec<String>>,
    filter: Option<String>,
) -> StellarHostsResult {
    // Define default columns if none specified
    let default_columns =
        vec!["hostname", "sy_dist", "st_teff", "st_mass", "sy_pnum"];
    get_table_data(
        df,
        all_metadata,
        page,
        limit,
        sort_by,
        order,
        selected_columns,
        filter,
        TableConfig {
            default_columns: &default_columns,
        },
    )
}

/// Get paginated exoplanets data with sorting
///
/// This is the core business logic that both server functions and REST handlers use.
/// It performs Polars operations and returns the data in a simple format.
///
/// # Arguments
/// * `df` - Reference to the exoplanets DataFrame
/// * `all_metadata` - Reference to all column metadata
/// * `page` - Page number (1-indexed)
/// * `limit` - Number of rows per page
/// * `sort_by` - Optional column name to sort by
/// * `order` - Sort order ("asc" or "desc")
/// * `selected_columns` - Optional list of column names to display
///
/// # Returns
/// A tuple of (rows as JSON, filtered_total, unfiltered_total, column names)
pub fn get_exoplanets_data(
    df: &DataFrame,
    all_metadata: &Arc<HashMap<String, ColumnMetadata>>,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    selected_columns: Option<Vec<String>>,
    filter: Option<String>,
) -> ExoplanetsResult {
    // Define default columns if none specified
    let default_columns = vec![
        "pl_name",
        "hostname",
        "discoverymethod",
        "disc_year",
        "pl_orbper",
        "pl_rade",
        "pl_bmasse",
    ];
    get_table_data(
        df,
        all_metadata,
        page,
        limit,
        sort_by,
        order,
        selected_columns,
        filter,
        TableConfig {
            default_columns: &default_columns,
        },
    )
}

fn table_result_from_cache_value(
    cached: TableCacheValue,
) -> (Vec<Value>, usize, usize, Vec<String>) {
    (cached.rows, cached.total, cached.total_all, cached.columns)
}

/// Cached variant of `get_stellarhosts_data`.
pub async fn get_stellarhosts_data_cached(
    df: &DataFrame,
    all_metadata: &Arc<HashMap<String, ColumnMetadata>>,
    table_cache: &TableCache,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    selected_columns: Option<Vec<String>>,
    filter: Option<String>,
) -> StellarHostsResult {
    let key = normalize_table_cache_key(
        TableKind::StellarHosts,
        page,
        limit,
        sort_by.clone(),
        order.clone(),
        selected_columns.clone(),
        filter.clone(),
    );

    if let Some(cached) = table_cache.get(&key).await {
        return Ok(table_result_from_cache_value(cached));
    }

    let df = df.clone();
    let all_metadata = Arc::clone(all_metadata);
    let (rows, total, total_all, columns) =
        tokio::task::spawn_blocking(move || {
            get_stellarhosts_data(
                &df,
                &all_metadata,
                page,
                limit,
                sort_by,
                order,
                selected_columns,
                filter,
            )
        })
        .await
        .map_err(|e| {
            format!("Failed to join stellarhosts blocking task: {}", e)
        })??;

    table_cache
        .insert(
            key,
            TableCacheValue {
                rows: rows.clone(),
                columns: columns.clone(),
                total,
                total_all,
            },
        )
        .await;

    Ok((rows, total, total_all, columns))
}

/// Cached variant of `get_exoplanets_data`.
pub async fn get_exoplanets_data_cached(
    df: &DataFrame,
    all_metadata: &Arc<HashMap<String, ColumnMetadata>>,
    table_cache: &TableCache,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    selected_columns: Option<Vec<String>>,
    filter: Option<String>,
) -> ExoplanetsResult {
    let key = normalize_table_cache_key(
        TableKind::Exoplanets,
        page,
        limit,
        sort_by.clone(),
        order.clone(),
        selected_columns.clone(),
        filter.clone(),
    );

    if let Some(cached) = table_cache.get(&key).await {
        return Ok(table_result_from_cache_value(cached));
    }

    let df = df.clone();
    let all_metadata = Arc::clone(all_metadata);
    let (rows, total, total_all, columns) =
        tokio::task::spawn_blocking(move || {
            get_exoplanets_data(
                &df,
                &all_metadata,
                page,
                limit,
                sort_by,
                order,
                selected_columns,
                filter,
            )
        })
        .await
        .map_err(|e| {
            format!("Failed to join exoplanets blocking task: {}", e)
        })??;

    table_cache
        .insert(
            key,
            TableCacheValue {
                rows: rows.clone(),
                columns: columns.clone(),
                total,
                total_all,
            },
        )
        .await;

    Ok((rows, total, total_all, columns))
}

/// Get all stellar host records for a given hostname
///
/// Returns all columns for each matching stellar host row.
pub fn get_stellar_host_by_name(
    df: &DataFrame,
    all_metadata: &Arc<HashMap<String, ColumnMetadata>>,
    hostname: &str,
) -> Result<(Vec<Value>, HashMap<String, ColumnMetadata>), String> {
    let filtered = df
        .clone()
        .lazy()
        .filter(col("hostname").eq(lit(hostname)))
        .collect()
        .map_err(|e| format!("Failed to filter by hostname: {}", e))?;

    if filtered.height() == 0 {
        return Err(format!("Stellar host '{}' not found", hostname));
    }

    let rows = dataframe_to_json(&filtered)?;

    Ok((rows, all_metadata.as_ref().clone()))
}

/// Get all exoplanet records for a given planet name
///
/// Returns all columns for each matching exoplanet row.
pub fn get_exoplanet_by_name(
    df: &DataFrame,
    all_metadata: &Arc<HashMap<String, ColumnMetadata>>,
    pl_name: &str,
) -> Result<(Vec<Value>, HashMap<String, ColumnMetadata>), String> {
    let filtered = df
        .clone()
        .lazy()
        .filter(col("pl_name").eq(lit(pl_name)))
        .collect()
        .map_err(|e| format!("Failed to filter by planet name: {}", e))?;

    if filtered.height() == 0 {
        return Err(format!("Exoplanet '{}' not found", pl_name));
    }

    let rows = dataframe_to_json(&filtered)?;

    Ok((rows, all_metadata.as_ref().clone()))
}

/// Get planets for a given hostname
///
/// Returns planets with selected columns for display.
pub fn get_planets_by_hostname(
    df: &DataFrame,
    all_metadata: &Arc<HashMap<String, ColumnMetadata>>,
    hostname: &str,
) -> Result<(Vec<Value>, Vec<String>, HashMap<String, ColumnMetadata>), String> {
    // Columns to show for planets in detail view
    let columns = vec![
        "pl_name",
        "discoverymethod",
        "disc_year",
        "pl_orbper",
        "pl_rade",
        "pl_bmasse",
        "pl_eqt",
    ];

    // Filter columns that exist in the dataframe
    let valid_columns: Vec<&str> = columns
        .iter()
        .filter(|c| df.column(c).is_ok())
        .copied()
        .collect();

    // Filter by hostname and select columns
    let filtered = df
        .clone()
        .lazy()
        .filter(col("hostname").eq(lit(hostname)))
        .select(valid_columns.iter().map(|c| col(*c)).collect::<Vec<_>>())
        .collect()
        .map_err(|e| format!("Failed to filter planets: {}", e))?;

    // Deduplicate by pl_name (exoplanets table has multiple rows per planet)
    let filtered = filtered
        .unique::<String, String>(
            Some(&["pl_name".to_string()]),
            UniqueKeepStrategy::First,
            None,
        )
        .map_err(|e| format!("Failed to deduplicate planets: {}", e))?;

    let rows = dataframe_to_json(&filtered)?;
    let column_names: Vec<String> =
        valid_columns.iter().map(|s| s.to_string()).collect();

    // Filter metadata to only include relevant columns
    let filtered_metadata: HashMap<String, ColumnMetadata> = all_metadata
        .iter()
        .filter(|(k, _)| valid_columns.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    Ok((rows, column_names, filtered_metadata))
}

/// Helper function to convert DataFrame to JSON
///
/// Converts each row of the DataFrame into a JSON object, handling different data types
/// and null values appropriately.
pub fn dataframe_to_json(df: &DataFrame) -> Result<Vec<Value>, String> {
    let mut rows = Vec::new();
    let columns = df.get_column_names();

    for row_idx in 0..df.height() {
        let mut row_map = serde_json::Map::new();

        for col_name in &columns {
            if let Ok(col) = df.column(col_name) {
                let value = match col.dtype() {
                    DataType::String => col
                        .str()
                        .map_err(|e| {
                            format!("Failed to get string column: {}", e)
                        })?
                        .get(row_idx)
                        .map(|s| json!(s))
                        .unwrap_or(json!(null)),
                    DataType::Float64 => col
                        .f64()
                        .map_err(|e| format!("Failed to get f64 column: {}", e))?
                        .get(row_idx)
                        .map(|f| json!(f))
                        .unwrap_or(json!(null)),
                    DataType::Float32 => col
                        .f32()
                        .map_err(|e| format!("Failed to get f32 column: {}", e))?
                        .get(row_idx)
                        .map(|f| json!(f))
                        .unwrap_or(json!(null)),
                    DataType::Int64 => col
                        .i64()
                        .map_err(|e| format!("Failed to get i64 column: {}", e))?
                        .get(row_idx)
                        .map(|i| json!(i))
                        .unwrap_or(json!(null)),
                    DataType::Int32 => col
                        .i32()
                        .map_err(|e| format!("Failed to get i32 column: {}", e))?
                        .get(row_idx)
                        .map(|i| json!(i))
                        .unwrap_or(json!(null)),
                    DataType::UInt32 => col
                        .u32()
                        .map_err(|e| format!("Failed to get u32 column: {}", e))?
                        .get(row_idx)
                        .map(|i| json!(i))
                        .unwrap_or(json!(null)),
                    DataType::UInt64 => col
                        .u64()
                        .map_err(|e| format!("Failed to get u64 column: {}", e))?
                        .get(row_idx)
                        .map(|i| json!(i))
                        .unwrap_or(json!(null)),
                    _ => json!(null),
                };
                row_map.insert(col_name.to_string(), value);
            }
        }

        rows.push(json!(row_map));
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::cache::{
        TableKind, build_table_cache, normalize_table_cache_key,
    };
    use polars::df;

    #[test]
    fn test_get_stellarhosts_data_pagination() {
        // Create test DataFrame
        let df = df! {
            "hostname" => &["Star A", "Star B", "Star C", "Star D", "Star E"],
            "sy_dist" => &[10.5, 20.3, 15.7, 8.2, 30.1],
            "st_teff" => &[5778.0, 6000.0, 5500.0, 5200.0, 6500.0],
            "st_mass" => &[1.0, 1.2, 0.9, 0.8, 1.5],
            "sy_pnum" => &[1, 2, 1, 3, 1],
        }
        .unwrap();

        // Create empty metadata for testing
        let metadata = Arc::new(HashMap::new());

        // Test first page
        let (rows, total, total_all, _cols) =
            get_stellarhosts_data(&df, &metadata, 1, 2, None, None, None, None)
                .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(total, 5);
        assert_eq!(total_all, 5);

        // Test second page
        let (rows, total, total_all, _cols) =
            get_stellarhosts_data(&df, &metadata, 2, 2, None, None, None, None)
                .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(total, 5);
        assert_eq!(total_all, 5);

        // Test last page (partial)
        let (rows, total, total_all, _cols) =
            get_stellarhosts_data(&df, &metadata, 3, 2, None, None, None, None)
                .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(total, 5);
        assert_eq!(total_all, 5);
    }

    #[test]
    fn test_get_stellarhosts_data_sorting() {
        let df = df! {
            "hostname" => &["Star C", "Star A", "Star B"],
            "sy_dist" => &[15.7, 10.5, 20.3],
            "st_teff" => &[5500.0, 5778.0, 6000.0],
            "st_mass" => &[0.9, 1.0, 1.2],
            "sy_pnum" => &[1, 1, 2],
        }
        .unwrap();

        // Create empty metadata for testing
        let metadata = Arc::new(HashMap::new());

        // Test ascending sort
        let (rows, _, _, _) = get_stellarhosts_data(
            &df,
            &metadata,
            1,
            10,
            Some("hostname".to_string()),
            Some("asc".to_string()),
            None,
            None,
        )
        .unwrap();
        assert_eq!(rows[0]["hostname"], "Star A");
        assert_eq!(rows[1]["hostname"], "Star B");
        assert_eq!(rows[2]["hostname"], "Star C");

        // Test descending sort
        let (rows, _, _, _) = get_stellarhosts_data(
            &df,
            &metadata,
            1,
            10,
            Some("sy_dist".to_string()),
            Some("desc".to_string()),
            None,
            None,
        )
        .unwrap();
        assert_eq!(rows[0]["sy_dist"], 20.3);
        assert_eq!(rows[1]["sy_dist"], 15.7);
        assert_eq!(rows[2]["sy_dist"], 10.5);
    }

    #[test]
    fn test_get_stellar_host_by_name_returns_all_rows() {
        let df = df! {
            "hostname" => &["Star A", "Star A", "Star B"],
            "sy_dist" => &[10.0, 10.1, 30.0],
            "st_teff" => &[5500.0, 5520.0, 6000.0],
        }
        .unwrap();

        let metadata = Arc::new(HashMap::new());

        let (rows, _meta) =
            get_stellar_host_by_name(&df, &metadata, "Star A").unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["hostname"], "Star A");
        assert_eq!(rows[1]["hostname"], "Star A");
    }

    #[test]
    fn test_get_exoplanet_by_name_returns_all_rows() {
        let df = df! {
            "pl_name" => &["Planet X", "Planet X", "Planet Y"],
            "hostname" => &["Star A", "Star A", "Star B"],
            "disc_year" => &[2010i64, 2012i64, 2018i64],
        }
        .unwrap();

        let metadata = Arc::new(HashMap::new());

        let (rows, _meta) =
            get_exoplanet_by_name(&df, &metadata, "Planet X").unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["pl_name"], "Planet X");
        assert_eq!(rows[1]["pl_name"], "Planet X");
    }

    #[test]
    fn test_column_filtering() {
        let df = df! {
            "hostname" => &["Star A", "Star B", "Star C"],
            "sy_dist" => &[10.5, 20.3, 15.7],
            "st_teff" => &[5778.0, 6000.0, 5500.0],
            "st_mass" => &[1.0, 1.2, 0.9],
            "sy_pnum" => &[1, 2, 1],
        }
        .unwrap();

        let metadata = Arc::new(HashMap::new());

        // Request only specific columns
        let selected_columns =
            vec!["hostname".to_string(), "sy_dist".to_string()];
        let (rows, _, _, columns) = get_stellarhosts_data(
            &df,
            &metadata,
            1,
            10,
            None,
            None,
            Some(selected_columns),
            None,
        )
        .unwrap();

        // Verify only requested columns are returned
        assert_eq!(columns.len(), 2);
        assert!(columns.contains(&"hostname".to_string()));
        assert!(columns.contains(&"sy_dist".to_string()));

        // Verify data contains only requested columns
        for row in &rows {
            assert!(row.get("hostname").is_some());
            assert!(row.get("sy_dist").is_some());
            assert!(row.get("st_teff").is_none());
            assert!(row.get("st_mass").is_none());
        }
    }

    #[test]
    fn test_invalid_column_filtering() {
        let df = df! {
            "hostname" => &["Star A", "Star B"],
            "sy_dist" => &[10.5, 20.3],
            "st_teff" => &[5778.0, 6000.0],
            "st_mass" => &[1.0, 1.2],
            "sy_pnum" => &[1, 2],
        }
        .unwrap();

        let metadata = Arc::new(HashMap::new());

        // Request invalid columns
        let selected_columns =
            vec!["invalid_col1".to_string(), "invalid_col2".to_string()];
        let result = get_stellarhosts_data(
            &df,
            &metadata,
            1,
            10,
            None,
            None,
            Some(selected_columns),
            None,
        );

        // Should return error for no valid columns
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No valid columns selected");
    }

    #[test]
    fn test_partial_valid_column_filtering() {
        let df = df! {
            "hostname" => &["Star A", "Star B"],
            "sy_dist" => &[10.5, 20.3],
            "st_teff" => &[5778.0, 6000.0],
            "st_mass" => &[1.0, 1.2],
            "sy_pnum" => &[1, 2],
        }
        .unwrap();

        let metadata = Arc::new(HashMap::new());

        // Request mix of valid and invalid columns
        let selected_columns = vec![
            "hostname".to_string(),
            "invalid_col".to_string(),
            "sy_dist".to_string(),
        ];
        let (_rows, _, _, columns) = get_stellarhosts_data(
            &df,
            &metadata,
            1,
            10,
            None,
            None,
            Some(selected_columns),
            None,
        )
        .unwrap();

        // Should return only valid columns
        assert_eq!(columns.len(), 2);
        assert!(columns.contains(&"hostname".to_string()));
        assert!(columns.contains(&"sy_dist".to_string()));
    }

    #[tokio::test]
    async fn test_cached_stellarhosts_miss_then_hit_same_normalized_key() {
        let df = df! {
            "hostname" => &["Star A", "Star B", "Star C"],
            "sy_dist" => &[10.5, 20.3, 15.7],
            "st_teff" => &[5778.0, 6000.0, 5500.0],
            "st_mass" => &[1.0, 1.2, 0.9],
            "sy_pnum" => &[1i64, 2i64, 1i64],
        }
        .unwrap();

        let metadata = Arc::new(HashMap::new());
        let table_cache = build_table_cache(64);
        let key = normalize_table_cache_key(
            TableKind::StellarHosts,
            1,
            50,
            None,
            None,
            None,
            None,
        );

        assert!(table_cache.get(&key).await.is_none());

        let first = get_stellarhosts_data_cached(
            &df,
            &metadata,
            &table_cache,
            0,
            50,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        assert!(table_cache.get(&key).await.is_some());

        let second = get_stellarhosts_data_cached(
            &df,
            &metadata,
            &table_cache,
            1,
            50,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(first.0, second.0);
        assert_eq!(first.1, second.1);
        assert_eq!(first.2, second.2);
        assert_eq!(first.3, second.3);
    }

    #[tokio::test]
    async fn test_table_cache_prewarm_populates_expected_default_keys() {
        let stellarhosts_df = df! {
            "hostname" => &["Star A", "Star B", "Star C"],
            "sy_dist" => &[10.5, 20.3, 15.7],
            "st_teff" => &[5778.0, 6000.0, 5500.0],
            "st_mass" => &[1.0, 1.2, 0.9],
            "sy_pnum" => &[1i64, 2i64, 1i64],
        }
        .unwrap();
        let exoplanets_df = df! {
            "pl_name" => &["Planet A", "Planet B", "Planet C"],
            "hostname" => &["Star A", "Star B", "Star C"],
            "discoverymethod" => &["Transit", "Transit", "Imaging"],
            "disc_year" => &[2010i64, 2011i64, 2012i64],
            "pl_orbper" => &[3.2, 4.1, 12.0],
            "pl_rade" => &[1.1, 2.2, 3.3],
            "pl_bmasse" => &[1.3, 2.4, 3.5],
        }
        .unwrap();

        let metadata = Arc::new(HashMap::new());
        let table_cache = build_table_cache(64);

        let stellarhosts_key = normalize_table_cache_key(
            TableKind::StellarHosts,
            1,
            50,
            None,
            None,
            None,
            None,
        );
        let exoplanets_key = normalize_table_cache_key(
            TableKind::Exoplanets,
            1,
            50,
            None,
            None,
            None,
            None,
        );

        assert!(table_cache.get(&stellarhosts_key).await.is_none());
        assert!(table_cache.get(&exoplanets_key).await.is_none());

        get_stellarhosts_data_cached(
            &stellarhosts_df,
            &metadata,
            &table_cache,
            1,
            50,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        get_exoplanets_data_cached(
            &exoplanets_df,
            &metadata,
            &table_cache,
            1,
            50,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        assert!(table_cache.get(&stellarhosts_key).await.is_some());
        assert!(table_cache.get(&exoplanets_key).await.is_some());
    }

    #[tokio::test]
    async fn test_cache_restart_semantics_fresh_cache_starts_empty() {
        let df = df! {
            "hostname" => &["Star A", "Star B"],
            "sy_dist" => &[10.5, 20.3],
            "st_teff" => &[5778.0, 6000.0],
            "st_mass" => &[1.0, 1.2],
            "sy_pnum" => &[1i64, 2i64],
        }
        .unwrap();
        let metadata = Arc::new(HashMap::new());

        let warmed_cache = build_table_cache(64);
        let key = normalize_table_cache_key(
            TableKind::StellarHosts,
            1,
            50,
            None,
            None,
            None,
            None,
        );

        get_stellarhosts_data_cached(
            &df,
            &metadata,
            &warmed_cache,
            1,
            50,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        assert!(warmed_cache.get(&key).await.is_some());

        let fresh_cache = build_table_cache(64);
        assert!(fresh_cache.get(&key).await.is_none());
    }
}
