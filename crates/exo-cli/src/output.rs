use anyhow::{Result, anyhow};
use clap::ValueEnum;
use comfy_table::{Table, presets::ASCII_MARKDOWN};
use polars::prelude::*;
use serde_json::Value;

pub use exo_core::json::dataframe_to_json;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
    Lines,
}

impl OutputFormat {
    pub fn from_config(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            "lines" => Ok(Self::Lines),
            _ => Err(anyhow!("output format must be table, json, csv, or lines")),
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
        OutputFormat::Lines => {
            for row in rows {
                let fields: Vec<String> = columns
                    .iter()
                    .filter_map(|column| {
                        let value = &row[column];
                        if value.is_null() {
                            None
                        } else {
                            Some(format_json_value(Some(value)))
                        }
                    })
                    .collect();
                println!("{}", fields.join("  "));
            }
            Ok(())
        }
    }
}

/// Compact per-system lines: the caller supplies the line formatter, one
/// line per row, safe for narrow terminals.
pub fn render_lines(rows: &[Value], line: impl Fn(&Value) -> String) {
    for row in rows {
        println!("{}", line(row));
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
