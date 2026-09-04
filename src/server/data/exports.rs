use std::collections::HashMap;

use polars::prelude::DataFrame;
use serde::Serialize;
use serde_json::Value;

use super::details;
use crate::metadata_helpers::encode_path_segment;
use crate::server::cache::HostDetailCache;
use crate::server::functions::{ExoplanetDetail, StellarHostDetail};
use exo_types::metadata::ColumnMetadata;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportEntity {
    StellarHost,
    Exoplanet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetailExport {
    pub filename: String,
    pub mime_type: &'static str,
    pub content: String,
    pub url: String,
}

impl ExportEntity {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "stellarhost" | "stellarhosts" => Ok(Self::StellarHost),
            "exoplanet" | "exoplanets" => Ok(Self::Exoplanet),
            _ => Err(format!(
                "entity must be 'stellarhost' or 'exoplanet', got '{value}'"
            )),
        }
    }

    fn route_prefix(self) -> &'static str {
        match self {
            Self::StellarHost => "stellarhosts",
            Self::Exoplanet => "exoplanets",
        }
    }

    fn filename_prefix(self) -> &'static str {
        match self {
            Self::StellarHost => "stellarhost",
            Self::Exoplanet => "exoplanet",
        }
    }
}

impl ExportFormat {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            _ => Err(format!("format must be 'json' or 'csv', got '{value}'")),
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
        }
    }

    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Json => "application/json; charset=utf-8",
            Self::Csv => "text/csv; charset=utf-8",
        }
    }
}

pub async fn export_stellarhost(
    df: &DataFrame,
    host_detail_cache: &HostDetailCache,
    metadata: &HashMap<String, ColumnMetadata>,
    site_url: &str,
    hostname: &str,
    format: ExportFormat,
) -> Result<DetailExport, String> {
    let content = match format {
        ExportFormat::Json => {
            let (detail, metadata) = details::get_stellar_host_detail_cached(
                df,
                host_detail_cache,
                metadata,
                hostname,
            )
            .await?;

            to_pretty_json(&StellarHostDetail { metadata, ..detail })?
        }
        ExportFormat::Csv => {
            let (rows, _) =
                details::get_stellar_host_by_name(df, metadata, hostname)?;
            rows_to_csv(&rows, &dataframe_columns(df))?
        }
    };

    Ok(build_export(
        ExportEntity::StellarHost,
        hostname,
        format,
        site_url,
        content,
    ))
}

pub fn export_exoplanet(
    df: &DataFrame,
    metadata: &HashMap<String, ColumnMetadata>,
    site_url: &str,
    pl_name: &str,
    format: ExportFormat,
) -> Result<DetailExport, String> {
    let content = match format {
        ExportFormat::Json => {
            let (records, metadata) =
                details::get_exoplanet_by_name(df, metadata, pl_name)?;
            let canonical =
                crate::server::exoplanet_canonical::build_canonical_exoplanet(
                    &records, &metadata,
                );
            to_pretty_json(&ExoplanetDetail {
                pl_name: pl_name.to_string(),
                canonical,
                records,
                metadata,
            })?
        }
        ExportFormat::Csv => {
            let (rows, _) =
                details::get_exoplanet_by_name(df, metadata, pl_name)?;
            rows_to_csv(&rows, &dataframe_columns(df))?
        }
    };

    Ok(build_export(
        ExportEntity::Exoplanet,
        pl_name,
        format,
        site_url,
        content,
    ))
}

pub fn parse_suffixed_name(value: &str, format: ExportFormat) -> Option<&str> {
    value.strip_suffix(&format!(".{}", format.extension()))
}

pub fn safe_filename(
    entity: ExportEntity,
    name: &str,
    format: ExportFormat,
) -> String {
    let safe_name = name
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' => ch,
            _ => '_',
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    let safe_name = if safe_name.is_empty() {
        "detail".to_string()
    } else {
        safe_name
    };

    format!(
        "{}-{}.{}",
        entity.filename_prefix(),
        safe_name,
        format.extension()
    )
}

fn build_export(
    entity: ExportEntity,
    name: &str,
    format: ExportFormat,
    site_url: &str,
    content: String,
) -> DetailExport {
    let site_url = site_url.trim_end_matches('/');

    DetailExport {
        filename: safe_filename(entity, name, format),
        mime_type: format.mime_type(),
        content,
        url: format!(
            "{site_url}/{}/{}.{}",
            entity.route_prefix(),
            encode_path_segment(name),
            format.extension()
        ),
    }
}

fn dataframe_columns(df: &DataFrame) -> Vec<String> {
    df.get_column_names()
        .into_iter()
        .map(ToString::to_string)
        .collect()
}

fn to_pretty_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value)
        .map_err(|error| format!("Failed to serialize JSON export: {error}"))
}

fn rows_to_csv(rows: &[Value], columns: &[String]) -> Result<String, String> {
    let mut output = String::new();
    output.push_str(
        &columns
            .iter()
            .map(|column| csv_escape(column))
            .collect::<Vec<_>>()
            .join(","),
    );
    output.push('\n');

    for row in rows {
        let object = row
            .as_object()
            .ok_or_else(|| "CSV export rows must be JSON objects".to_string())?;
        let values = columns
            .iter()
            .map(|column| {
                let value = object.get(column).unwrap_or(&Value::Null);
                csv_escape(&csv_value(value))
            })
            .collect::<Vec<_>>();
        output.push_str(&values.join(","));
        output.push('\n');
    }

    Ok(output)
}

fn csv_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn safe_filename_replaces_path_unsafe_characters() {
        assert_eq!(
            safe_filename(
                ExportEntity::Exoplanet,
                "Kepler-10 b/alpha",
                ExportFormat::Json
            ),
            "exoplanet-Kepler-10_b_alpha.json"
        );
    }

    #[test]
    fn parse_suffixed_name_strips_only_matching_final_suffix() {
        assert_eq!(
            parse_suffixed_name("Kepler.10 b.json", ExportFormat::Json),
            Some("Kepler.10 b")
        );
        assert_eq!(
            parse_suffixed_name("Kepler.10 b.csv", ExportFormat::Json),
            None
        );
    }

    #[test]
    fn rows_to_csv_uses_column_order_and_escapes_values() {
        let rows = vec![json!({
            "name": "Kepler, \"Ten\"",
            "mass": 3.15,
            "missing": null,
        })];
        let columns = vec![
            "name".to_string(),
            "mass".to_string(),
            "missing".to_string(),
        ];

        let csv = rows_to_csv(&rows, &columns).unwrap();

        assert_eq!(csv, "name,mass,missing\n\"Kepler, \"\"Ten\"\"\",3.15,\n");
    }
}
