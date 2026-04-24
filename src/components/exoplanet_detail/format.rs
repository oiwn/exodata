use std::collections::HashMap;

use leptos::serde_json::Value;

use crate::server::functions::ColumnMetadata;

pub fn format_value(value: &Value, unit: &str) -> String {
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

pub fn property_display(
    record: &Value,
    metadata: &HashMap<String, ColumnMetadata>,
    key: &str,
    fallback_unit: &str,
) -> String {
    let value = record.get(key).unwrap_or(&Value::Null);
    let unit = metadata
        .get(key)
        .and_then(|meta| meta.unit.as_deref())
        .unwrap_or(fallback_unit);

    format_value(value, unit)
}

pub fn first_non_empty_string(records: &[Value], key: &str) -> Option<String> {
    records.iter().find_map(|record| {
        record
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    })
}

pub fn median_numeric_value(records: &[Value], key: &str) -> Option<f64> {
    let mut values = records
        .iter()
        .filter_map(|record| record.get(key).and_then(json_number_to_f64))
        .collect::<Vec<_>>();

    if values.is_empty() {
        return None;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;

    if values.len() % 2 == 0 {
        Some((values[mid - 1] + values[mid]) / 2.0)
    } else {
        Some(values[mid])
    }
}

pub fn planet_visual_class(
    radius_rearth: Option<f64>,
    equilibrium_temp: Option<f64>,
) -> &'static str {
    match (radius_rearth, equilibrium_temp) {
        (_, Some(temp)) if temp >= 1200.0 => "planet-visual--hot",
        (Some(radius), _) if radius >= 8.0 => "planet-visual--giant",
        (Some(radius), _) if radius >= 3.0 => "planet-visual--sub-neptune",
        (_, Some(temp)) if temp <= 180.0 => "planet-visual--cold",
        _ => "planet-visual--temperate",
    }
}

pub fn comparison_scale(radius_rearth: f64, max_radius_rearth: f64) -> f64 {
    if max_radius_rearth <= 0.0 {
        0.0
    } else {
        radius_rearth / max_radius_rearth
    }
}

fn json_number_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .or_else(|| number.as_i64().map(|value| value as f64))
            .or_else(|| number.as_u64().map(|value| value as f64)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::serde_json::json;

    #[test]
    fn format_number_handles_integer_decimal_and_scientific_ranges() {
        assert_eq!(format_number(12.0), "12");
        assert_eq!(format_number(12.345), "12.35");
        assert_eq!(format_number(345.67), "345.7");
        assert_eq!(format_number(0.0012), "1.20e-3");
    }

    #[test]
    fn format_value_applies_units_and_bool_labels() {
        assert_eq!(format_value(&json!(2.5), "R⊕"), "2.50 R⊕");
        assert_eq!(format_value(&json!(true), ""), "Yes");
        assert_eq!(format_value(&Value::Null, ""), "—");
    }

    #[test]
    fn first_non_empty_string_skips_null_and_blank_values() {
        let records = vec![
            json!({ "hostname": null }),
            json!({ "hostname": "   " }),
            json!({ "hostname": "Kepler-22" }),
        ];

        assert_eq!(
            first_non_empty_string(&records, "hostname"),
            Some("Kepler-22".to_string())
        );
    }

    #[test]
    fn median_numeric_value_uses_middle_or_average_of_two_values() {
        let odd = vec![
            json!({ "pl_rade": 1.0 }),
            json!({ "pl_rade": 3.0 }),
            json!({ "pl_rade": 2.0 }),
        ];
        let even = vec![
            json!({ "pl_rade": 1.0 }),
            json!({ "pl_rade": 2.0 }),
            json!({ "pl_rade": 4.0 }),
            json!({ "pl_rade": 10.0 }),
        ];

        assert_eq!(median_numeric_value(&odd, "pl_rade"), Some(2.0));
        assert_eq!(median_numeric_value(&even, "pl_rade"), Some(3.0));
    }

    #[test]
    fn planet_visual_class_prefers_temperature_then_size_bands() {
        assert_eq!(
            planet_visual_class(Some(2.0), Some(1300.0)),
            "planet-visual--hot"
        );
        assert_eq!(
            planet_visual_class(Some(9.0), Some(400.0)),
            "planet-visual--giant"
        );
        assert_eq!(
            planet_visual_class(Some(4.0), None),
            "planet-visual--sub-neptune"
        );
        assert_eq!(
            planet_visual_class(None, Some(150.0)),
            "planet-visual--cold"
        );
        assert_eq!(
            planet_visual_class(Some(1.0), Some(300.0)),
            "planet-visual--temperate"
        );
    }

    #[test]
    fn comparison_scale_is_linear_and_handles_zero_denominator() {
        assert_eq!(comparison_scale(2.8, 11.2), 0.25);
        assert_eq!(comparison_scale(1.0, 0.0), 0.0);
    }
}
