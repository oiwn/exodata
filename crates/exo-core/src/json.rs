//! DataFrame serialization into JSON row objects.
use anyhow::{Result, anyhow};
use polars::prelude::{AnyValue, DataFrame};
use serde_json::{Map, Value, json};

pub fn dataframe_to_json(frame: &DataFrame) -> Result<Vec<Value>> {
    let columns = frame
        .get_column_names()
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    let mut rows = Vec::with_capacity(frame.height());

    for row_idx in 0..frame.height() {
        let mut object = Map::new();
        for column in &columns {
            let value = frame
                .column(column)?
                .get(row_idx)
                .map_err(|err| anyhow!("failed to read {}: {}", column, err))?;
            object.insert(column.clone(), any_value_to_json(value));
        }
        rows.push(Value::Object(object));
    }

    Ok(rows)
}

fn any_value_to_json(value: AnyValue<'_>) -> Value {
    match value {
        AnyValue::Null => Value::Null,
        AnyValue::Boolean(value) => json!(value),
        AnyValue::String(value) => json!(value),
        AnyValue::StringOwned(value) => json!(value.to_string()),
        AnyValue::Float64(value) if value.is_finite() => json!(value),
        AnyValue::Float32(value) if value.is_finite() => json!(value),
        AnyValue::Int64(value) => json!(value),
        AnyValue::Int32(value) => json!(value),
        AnyValue::Int16(value) => json!(value),
        AnyValue::Int8(value) => json!(value),
        AnyValue::UInt64(value) => json!(value),
        AnyValue::UInt32(value) => json!(value),
        AnyValue::UInt16(value) => json!(value),
        AnyValue::UInt8(value) => json!(value),
        other => json!(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::df;
    use serde_json::json;

    #[test]
    fn dataframe_to_json_converts_common_any_values() {
        let frame = df! {
            "name" => &[Some("Kepler-10"), None],
            "confirmed" => &[Some(true), None],
            "mass" => &[Some(3.33_f64), None],
            "radius" => &[Some(1.47_f32), None],
            "i64_col" => &[Some(-2_i64), None],
            "i32_col" => &[Some(-1_i32), None],
            "u64_col" => &[Some(2_u64), None],
            "u32_col" => &[Some(1_u32), None],
        }
        .unwrap();

        let rows = dataframe_to_json(&frame).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["name"], json!("Kepler-10"));
        assert_eq!(rows[0]["confirmed"], json!(true));
        assert_eq!(rows[0]["mass"], json!(3.33));
        assert_eq!(rows[0]["radius"], json!(1.47_f32));
        assert_eq!(rows[0]["i64_col"], json!(-2));
        assert_eq!(rows[0]["i32_col"], json!(-1));
        assert_eq!(rows[0]["u64_col"], json!(2));
        assert_eq!(rows[0]["u32_col"], json!(1));
        assert!(rows[1]["name"].is_null());
    }
}
