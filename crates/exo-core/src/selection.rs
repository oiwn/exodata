//! Select complete source rows, never synthesized measurements.
use serde_json::Value;
use std::collections::BTreeMap;

pub const HOST_SUMMARY_FIELDS: &[&str] = &[
    "st_spectype",
    "st_teff",
    "st_mass",
    "st_rad",
    "st_age",
    "st_lum",
    "st_met",
    "st_logg",
    "sy_dist",
    "sy_plx",
];

pub fn stellar_host_record_index(records: &[Value]) -> Option<usize> {
    records
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            let score = HOST_SUMMARY_FIELDS
                .iter()
                .filter(|&&field| {
                    if field == "st_spectype" {
                        row[field].as_str().is_some_and(|s| !s.trim().is_empty())
                    } else {
                        row[field].as_f64().is_some_and(f64::is_finite)
                    }
                })
                .count();
            (score > 0).then(|| {
                (
                    index,
                    score,
                    (
                        row["st_refname"].as_str().unwrap_or_default().to_owned(),
                        row["sy_refname"].as_str().unwrap_or_default().to_owned(),
                        canonical_json(row),
                    ),
                )
            })
        })
        .min_by(|a, b| b.1.cmp(&a.1).then_with(|| a.2.cmp(&b.2)))
        .map(|(index, _, _)| index)
}

pub fn exoplanet_record_index(records: &[Value]) -> Option<usize> {
    let mut defaults = records
        .iter()
        .enumerate()
        .filter(|(_, row)| row["default_flag"].as_i64() == Some(1));
    let (index, _) = defaults.next()?;
    defaults.next().is_none().then_some(index)
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<_, _> = map.iter().collect();
            format!(
                "{{{}}}",
                sorted
                    .into_iter()
                    .map(|(key, value)| {
                        format!(
                            "{}:{}",
                            serde_json::to_string(key).unwrap(),
                            canonical_json(value)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn host_scores_only_summary_measurements_and_preserves_source_row() {
        let rows = vec![
            json!({"st_mass": 1.0, "st_refname": "A", "st_masserr1": 0.1, "sy_vmag": 10, "hostname": "X"}),
            json!({"st_mass": 2.0, "st_teff": 0.0, "st_spectype": " "}),
        ];
        assert_eq!(stellar_host_record_index(&rows), Some(1));
        assert_eq!(rows[1]["st_mass"], 2.0);
        assert_eq!(
            stellar_host_record_index(&[
                json!({"st_teff": null, "st_mass": "2", "st_spectype": " "})
            ]),
            None
        );
        assert_eq!(
            stellar_host_record_index(&[json!({"st_teff": f64::INFINITY})]),
            None
        );
        assert_eq!(
            stellar_host_record_index(&[json!({"st_spectype": "G2"})]),
            Some(0)
        );
    }

    #[test]
    fn host_ties_are_independent_of_row_order_and_object_key_order() {
        let mut rows = vec![
            json!({"st_mass": 3.0, "st_refname": "B"}),
            json!({"st_mass": 2.0, "st_refname": "A", "sy_refname": "A"}),
            json!({"st_mass": 1.0, "st_refname": "A", "sy_refname": "A"}),
            json!({"st_mass": 0.0, "st_refname": "A", "sy_refname": "B"}),
        ];
        let selected = rows[stellar_host_record_index(&rows).unwrap()].clone();
        rows.reverse();
        assert_eq!(rows[stellar_host_record_index(&rows).unwrap()], selected);
        assert_eq!(selected["st_mass"], 1.0);
        assert_eq!(
            canonical_json(&json!({"b": {"z": 1, "a": 2}, "a": 0})),
            "{\"a\":0,\"b\":{\"a\":2,\"z\":1}}"
        );
    }

    #[test]
    fn planets_require_a_unique_default() {
        assert_eq!(exoplanet_record_index(&[]), None);
        assert_eq!(exoplanet_record_index(&[json!({"default_flag": 0})]), None);
        let default = json!({"default_flag": 1, "pl_rade": null});
        assert_eq!(
            exoplanet_record_index(&[json!({"pl_rade": 3}), default.clone()]),
            Some(1)
        );
        assert_eq!(exoplanet_record_index(&[default.clone(), default]), None);
    }
}
