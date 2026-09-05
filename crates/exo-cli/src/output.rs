use anyhow::{Result, anyhow};
use clap::ValueEnum;
use comfy_table::{Table, presets::ASCII_MARKDOWN};
use polars::prelude::*;
use serde_json::{Map, Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

impl OutputFormat {
    pub fn from_config(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            _ => Err(anyhow!("output format must be table, json, or csv")),
        }
    }
}

pub fn render_rows(
    rows: &[Value],
    columns: &[String],
    format: OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Table => render_json_table(rows, columns),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(rows)?);
            Ok(())
        }
        OutputFormat::Csv => render_json_csv(rows, columns),
    }
}

pub fn render_dataframe(frame: &DataFrame, format: OutputFormat) -> Result<()> {
    let columns = frame
        .get_column_names()
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    let rows = dataframe_to_json(frame)?;
    render_rows(&rows, &columns, format)
}

fn render_json_table(rows: &[Value], columns: &[String]) -> Result<()> {
    let mut table = Table::new();
    table.load_style(ASCII_MARKDOWN);
    table.set_header(columns.to_vec());

    for row in rows {
        let object = row
            .as_object()
            .ok_or_else(|| anyhow!("expected row object"))?;
        table.add_row(
            columns
                .iter()
                .map(|column| format_json_value(object.get(column)))
                .collect::<Vec<_>>(),
        );
    }

    println!("{}", table);
    Ok(())
}

fn render_json_csv(rows: &[Value], columns: &[String]) -> Result<()> {
    let mut writer = csv::Writer::from_writer(std::io::stdout());
    writer.write_record(columns)?;

    for row in rows {
        let object = row
            .as_object()
            .ok_or_else(|| anyhow!("expected row object"))?;
        let record = columns
            .iter()
            .map(|column| format_json_value(object.get(column)))
            .collect::<Vec<_>>();
        writer.write_record(record)?;
    }

    writer.flush()?;
    Ok(())
}

fn format_json_value(value: Option<&Value>) -> String {
    match value {
        Some(Value::Null) | None => String::new(),
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        Some(other) => other.to_string(),
    }
}

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
    fn output_format_from_config_accepts_known_values() {
        assert_eq!(
            OutputFormat::from_config("table").unwrap(),
            OutputFormat::Table
        );
        assert_eq!(
            OutputFormat::from_config(" JSON ").unwrap(),
            OutputFormat::Json
        );
        assert_eq!(OutputFormat::from_config("csv").unwrap(), OutputFormat::Csv);
        assert!(OutputFormat::from_config("yaml").is_err());
    }

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
