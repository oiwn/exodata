// Common business logic for server operations
// This module contains pure functions that are called by both Leptos server functions
// and Axum REST handlers. It's server-only and contains no HTTP/Leptos dependencies.

use polars::prelude::*;
use serde_json::{json, Value};

/// Result type for stellarhosts data operations
pub type StellarHostsResult = Result<(Vec<Value>, usize, Vec<String>), String>;

/// Get paginated stellar hosts data with sorting
///
/// This is the core business logic that both server functions and REST handlers use.
/// It performs Polars operations and returns the data in a simple format.
///
/// # Arguments
/// * `df` - Reference to the stellarhosts DataFrame
/// * `page` - Page number (1-indexed)
/// * `limit` - Number of rows per page
/// * `sort_by` - Optional column name to sort by
/// * `order` - Sort order ("asc" or "desc")
///
/// # Returns
/// A tuple of (rows as JSON, total count, column names)
pub fn get_stellarhosts_data(
    df: &DataFrame,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
) -> StellarHostsResult {
    // Clone the dataframe to work with it
    let mut df = df.clone();

    // Define the columns we want to display
    let columns_to_select = ["hostname", "sy_dist", "st_teff", "st_mass", "sy_pnum"];

    // Select only the columns we need
    df = df
        .select(columns_to_select)
        .map_err(|e| format!("Failed to select columns: {}", e))?;

    // Get total count before pagination
    let total = df.height();

    // Apply sorting if requested
    if let Some(sort_col) = &sort_by {
        let descending = order.as_deref().unwrap_or("asc") == "desc";
        let options = SortMultipleOptions::new().with_order_descending(descending);

        df = df
            .sort([sort_col.as_str()], options)
            .map_err(|e| format!("Failed to sort: {}", e))?;
    }

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
    let columns: Vec<String> = columns_to_select.iter().map(|s| (*s).to_string()).collect();

    Ok((rows, total, columns))
}

/// Helper function to convert DataFrame to JSON
///
/// Converts each row of the DataFrame into a JSON object, handling different data types
/// and null values appropriately.
fn dataframe_to_json(df: &DataFrame) -> Result<Vec<Value>, String> {
    let mut rows = Vec::new();
    let columns = df.get_column_names();

    for row_idx in 0..df.height() {
        let mut row_map = serde_json::Map::new();

        for col_name in &columns {
            if let Ok(col) = df.column(col_name) {
                let value = match col.dtype() {
                    DataType::String => {
                        col.str()
                            .map_err(|e| format!("Failed to get string column: {}", e))?
                            .get(row_idx)
                            .map(|s| json!(s))
                            .unwrap_or(json!(null))
                    }
                    DataType::Float64 => {
                        col.f64()
                            .map_err(|e| format!("Failed to get f64 column: {}", e))?
                            .get(row_idx)
                            .map(|f| json!(f))
                            .unwrap_or(json!(null))
                    }
                    DataType::Int64 => {
                        col.i64()
                            .map_err(|e| format!("Failed to get i64 column: {}", e))?
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

        // Test first page
        let (rows, total, _cols) = get_stellarhosts_data(&df, 1, 2, None, None).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(total, 5);

        // Test second page
        let (rows, total, _cols) = get_stellarhosts_data(&df, 2, 2, None, None).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(total, 5);

        // Test last page (partial)
        let (rows, total, _cols) = get_stellarhosts_data(&df, 3, 2, None, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(total, 5);
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

        // Test ascending sort
        let (rows, _, _) = get_stellarhosts_data(&df, 1, 10, Some("hostname".to_string()), Some("asc".to_string())).unwrap();
        assert_eq!(rows[0]["hostname"], "Star A");
        assert_eq!(rows[1]["hostname"], "Star B");
        assert_eq!(rows[2]["hostname"], "Star C");

        // Test descending sort
        let (rows, _, _) = get_stellarhosts_data(&df, 1, 10, Some("sy_dist".to_string()), Some("desc".to_string())).unwrap();
        assert_eq!(rows[0]["sy_dist"], 20.3);
        assert_eq!(rows[1]["sy_dist"], 15.7);
        assert_eq!(rows[2]["sy_dist"], 10.5);
    }
}
