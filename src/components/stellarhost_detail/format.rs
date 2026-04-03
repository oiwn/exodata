use leptos::serde_json::Value;

use crate::server::functions::{NumericFieldSummary, StableValueSummary};

pub fn alias_label(key: &str) -> &'static str {
    match key {
        "hd_name" => "HD",
        "hip_name" => "HIP",
        "tic_id" => "TIC",
        _ => "Alias",
    }
}

pub fn column_label(key: &str) -> String {
    match key {
        "st_refname" => "Stellar Ref".to_string(),
        "sy_refname" => "System Ref".to_string(),
        _ => key.replace('_', " ").to_uppercase(),
    }
}

pub fn format_numeric_primary(summary: &NumericFieldSummary) -> String {
    let number = format_number(summary.value);
    match summary.unit.as_deref() {
        Some(unit) if !unit.is_empty() => format!("{number} {unit}"),
        _ => number,
    }
}

pub fn format_json_value(value: &Value, unit: &str) -> String {
    match value {
        Value::Null => "—".to_string(),
        Value::String(text) => text.clone(),
        Value::Number(number) => {
            let formatted = if let Some(value) = number.as_f64() {
                format_number(value)
            } else if let Some(value) = number.as_i64() {
                value.to_string()
            } else if let Some(value) = number.as_u64() {
                value.to_string()
            } else {
                number.to_string()
            };

            if unit.is_empty() {
                formatted
            } else {
                format!("{formatted} {unit}")
            }
        }
        Value::Bool(value) => {
            if *value {
                "Yes".to_string()
            } else {
                "No".to_string()
            }
        }
        _ => value.to_string(),
    }
}

pub fn format_stable_value(summary: &StableValueSummary) -> String {
    format_json_value(&summary.value, summary.unit.as_deref().unwrap_or(""))
}

pub fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else if value.abs() < 0.01 || value.abs() >= 10000.0 {
        format!("{value:.2e}")
    } else if value.abs() >= 100.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    }
}
