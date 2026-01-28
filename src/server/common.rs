// Common business logic for server operations
// This module contains pure functions that are called by both Leptos server functions
// and Axum REST handlers. It's server-only and contains no HTTP/Leptos dependencies.

use exo_core::metadata::ColumnMetadata;
use polars::prelude::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

/// Result type for stellarhosts data operations
/// Returns: (rows, filtered_total, unfiltered_total, columns, metadata)
pub type StellarHostsResult = Result<
    (
        Vec<Value>,
        usize,
        usize,
        Vec<String>,
        HashMap<String, ColumnMetadata>,
    ),
    String,
>;

/// Result type for exoplanets data operations
/// Returns: (rows, filtered_total, unfiltered_total, columns, metadata)
pub type ExoplanetsResult = Result<
    (
        Vec<Value>,
        usize,
        usize,
        Vec<String>,
        HashMap<String, ColumnMetadata>,
    ),
    String,
>;

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
/// A tuple of (rows as JSON, filtered_total, unfiltered_total, column names, column metadata)
pub fn get_stellarhosts_data(
    df: &DataFrame,
    all_metadata: &Arc<HashMap<String, ColumnMetadata>>,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    selected_columns: Option<Vec<String>>,
) -> StellarHostsResult {
    // Clone the dataframe to work with it
    let mut df = df.clone();

    // Define default columns if none specified
    let default_columns =
        vec!["hostname", "sy_dist", "st_teff", "st_mass", "sy_pnum"];

    // Use selected columns or fall back to defaults
    let columns_to_select: Vec<&str> = if let Some(cols) = &selected_columns {
        // Validate that requested columns exist in dataframe
        cols.iter()
            .filter(|col| df.column(col).is_ok())
            .map(|s| s.as_str())
            .collect()
    } else {
        default_columns.iter().map(|s| *s).collect()
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

    // Return ALL metadata (not just selected columns) so the column selector
    // can show all available columns to the user
    let all_column_metadata: HashMap<String, ColumnMetadata> =
        all_metadata.as_ref().clone();

    Ok((rows, total, total_all, columns, all_column_metadata))
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
/// A tuple of (rows as JSON, filtered_total, unfiltered_total, column names, column metadata)
pub fn get_exoplanets_data(
    df: &DataFrame,
    all_metadata: &Arc<HashMap<String, ColumnMetadata>>,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    selected_columns: Option<Vec<String>>,
) -> ExoplanetsResult {
    // Clone the dataframe to work with it
    let mut df = df.clone();

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

    // Use selected columns or fall back to defaults
    let columns_to_select: Vec<&str> = if let Some(cols) = &selected_columns {
        // Validate that requested columns exist in dataframe
        cols.iter()
            .filter(|col| df.column(col).is_ok())
            .map(|s| s.as_str())
            .collect()
    } else {
        default_columns.iter().map(|s| *s).collect()
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

    // Return ALL metadata (not just selected columns) so the column selector
    // can show all available columns to the user
    let all_column_metadata: HashMap<String, ColumnMetadata> =
        all_metadata.as_ref().clone();

    Ok((rows, total, total_all, columns, all_column_metadata))
}

/// Get a single stellar host by hostname
///
/// Returns all columns for the matching stellar host.
pub fn get_stellar_host_by_name(
    df: &DataFrame,
    all_metadata: &Arc<HashMap<String, ColumnMetadata>>,
    hostname: &str,
) -> Result<(HashMap<String, Value>, HashMap<String, ColumnMetadata>), String> {
    // Filter by hostname
    let filtered = df
        .clone()
        .lazy()
        .filter(col("hostname").eq(lit(hostname)))
        .collect()
        .map_err(|e| format!("Failed to filter by hostname: {}", e))?;

    if filtered.height() == 0 {
        return Err(format!("Stellar host '{}' not found", hostname));
    }

    // Convert the single row to a map of properties
    let rows = dataframe_to_json(&filtered)?;
    let properties = rows
        .into_iter()
        .next()
        .and_then(|v| v.as_object().cloned())
        .map(|m| m.into_iter().collect::<HashMap<String, Value>>())
        .unwrap_or_default();

    Ok((properties, all_metadata.as_ref().clone()))
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
        let (rows, total, total_all, _cols, _meta) =
            get_stellarhosts_data(&df, &metadata, 1, 2, None, None, None)
                .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(total, 5);
        assert_eq!(total_all, 5);

        // Test second page
        let (rows, total, total_all, _cols, _meta) =
            get_stellarhosts_data(&df, &metadata, 2, 2, None, None, None)
                .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(total, 5);
        assert_eq!(total_all, 5);

        // Test last page (partial)
        let (rows, total, total_all, _cols, _meta) =
            get_stellarhosts_data(&df, &metadata, 3, 2, None, None, None)
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
        let (rows, _, _, _, _) = get_stellarhosts_data(
            &df,
            &metadata,
            1,
            10,
            Some("hostname".to_string()),
            Some("asc".to_string()),
            None,
        )
        .unwrap();
        assert_eq!(rows[0]["hostname"], "Star A");
        assert_eq!(rows[1]["hostname"], "Star B");
        assert_eq!(rows[2]["hostname"], "Star C");

        // Test descending sort
        let (rows, _, _, _, _) = get_stellarhosts_data(
            &df,
            &metadata,
            1,
            10,
            Some("sy_dist".to_string()),
            Some("desc".to_string()),
            None,
        )
        .unwrap();
        assert_eq!(rows[0]["sy_dist"], 20.3);
        assert_eq!(rows[1]["sy_dist"], 15.7);
        assert_eq!(rows[2]["sy_dist"], 10.5);
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
        let (rows, _, _, columns, _) = get_stellarhosts_data(
            &df,
            &metadata,
            1,
            10,
            None,
            None,
            Some(selected_columns),
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
        let (_rows, _, _, columns, _) = get_stellarhosts_data(
            &df,
            &metadata,
            1,
            10,
            None,
            None,
            Some(selected_columns),
        )
        .unwrap();

        // Should return only valid columns
        assert_eq!(columns.len(), 2);
        assert!(columns.contains(&"hostname".to_string()));
        assert!(columns.contains(&"sy_dist".to_string()));
    }
}
