use polars::prelude::*;
use serde_json::{Value, json};

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

pub fn distinct_value_count(
    df: &DataFrame,
    column: &str,
) -> Result<usize, String> {
    let distinct = df
        .clone()
        .unique::<String, String>(
            Some(&[column.to_string()]),
            UniqueKeepStrategy::First,
            None,
        )
        .map_err(|e| {
            format!("Failed to count distinct values for {column}: {e}")
        })?;

    Ok(distinct.height())
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::df;
    use serde_json::json;

    #[test]
    fn dataframe_to_json_handles_supported_types_and_nulls() {
        let df = df! {
            "name" => &[Some("Kepler-10"), None],
            "mass" => &[Some(1.4_f64), None],
            "radius" => &[Some(1.1_f32), None],
            "year64" => &[Some(2011_i64), None],
            "year32" => &[Some(2011_i32), None],
            "count32" => &[Some(2_u32), None],
            "count64" => &[Some(3_u64), None],
        }
        .unwrap();

        let rows = dataframe_to_json(&df).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["name"], json!("Kepler-10"));
        assert_eq!(rows[0]["mass"], json!(1.4));
        assert_eq!(rows[0]["radius"], json!(1.1_f32));
        assert_eq!(rows[0]["year64"], json!(2011));
        assert_eq!(rows[0]["year32"], json!(2011));
        assert_eq!(rows[0]["count32"], json!(2));
        assert_eq!(rows[0]["count64"], json!(3));
        assert!(rows[1]["name"].is_null());
        assert!(rows[1]["mass"].is_null());
    }

    #[test]
    fn distinct_value_count_counts_unique_column_values() {
        let df = df! {
            "hostname" => &["A", "A", "B", "C"],
        }
        .unwrap();

        assert_eq!(distinct_value_count(&df, "hostname").unwrap(), 3);
    }

    #[test]
    fn distinct_value_count_reports_missing_column() {
        let df = df! {
            "hostname" => &["A"],
        }
        .unwrap();

        let error = distinct_value_count(&df, "missing").unwrap_err();
        assert!(error.contains("missing"));
    }
}
