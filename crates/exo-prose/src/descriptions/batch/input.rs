//! Compact reader-facing facts. Saved preparation remains the audit record.
use serde_json::{Map, Value, json};

fn fields(value: &Value, names: &[&str]) -> Map<String, Value> {
    names
        .iter()
        .filter_map(|name| {
            value.get(*name).map(|v| ((*name).to_owned(), v.clone()))
        })
        .collect()
}

fn measurements(value: &Value) -> Value {
    let mut out = Map::new();
    if let Some(values) = value.as_object() {
        for (name, measurement) in values {
            let Some(display) = measurement["display"].as_str() else {
                continue;
            };
            let display = display
                .strip_prefix("planet mass (not a minimum-mass quantity): ")
                .or_else(|| {
                    display.strip_prefix("minimum-mass quantity (M sin i): ")
                })
                .unwrap_or(display);
            let key = if name == "mass"
                && measurement["provenance"].as_str() == Some("Msini")
            {
                "minimum_mass"
            } else {
                name
            };
            out.insert(
                key.to_owned(),
                json!(super::super::validate::remove_measurement_about(display)),
            );
        }
    }
    Value::Object(out)
}

pub(super) fn project(request: &Value) -> Value {
    let mut system = fields(
        &request["system"],
        &[
            "hostname",
            "matching_host_planet_count",
            "catalog_system_star_count",
        ],
    );
    if let Some(distance) = request["system"]["distance"]["display"].as_str() {
        system.insert(
            "distance".into(),
            json!(super::super::validate::remove_measurement_about(distance)),
        );
    }
    let mut star = fields(
        &request["star"],
        &["spectral_type", "host_kind", "age_class"],
    );
    star.insert(
        "measurements".into(),
        measurements(&request["star"]["measurements"]),
    );
    let planets: Vec<Value> = request["planets"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|planet| {
            let mut item = fields(
                planet,
                &[
                    "name",
                    "classification",
                    "circumbinary",
                    "discovery_method",
                    "discovery_year",
                ],
            );
            item.insert(
                "measurements".into(),
                measurements(&planet["measurements"]),
            );
            Value::Object(item)
        })
        .collect();
    let mut guide = request["guide"].as_object().cloned().unwrap_or_default();
    if guide.contains_key("minimum_mass") {
        guide.insert("minimum_mass".into(), json!("A minimum mass is a lower bound on a planet's true mass. It differs from an ordinary mass estimate."));
    }
    for entry in guide.values_mut() {
        if let Some(text) = entry.as_str() {
            *entry =
                json!(super::super::validate::remove_measurement_about(text));
        }
    }
    let comparisons: Vec<String> = request["publishable_comparisons"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        // Old requests and merged notes may still carry retired comparisons.
        .filter(|text| !super::super::validate::is_earth_year_reference(text))
        .map(|text| {
            super::super::validate::remove_measurement_about(
                &text.replace("reported minimum-mass quantity", "minimum mass"),
            )
        })
        .collect();
    let mut result = json!({"system": system, "star": star, "planets": planets,
        "publishable_comparisons": comparisons, "guide": guide});
    if let Some(constraints) = request.get("silent_constraints") {
        result["silent_constraints"] = constraints.clone();
    }
    if let Some(target) = request["request"].get("target_words") {
        result["target_words"] = target.clone();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_keeps_meaning_and_omits_audit_data() {
        let request = json!({"system": {"hostname": "A", "matching_host_planet_count": 2, "catalog_system_planet_count": 7, "catalog_system_star_count": 2},
            "star": {"measurements": {"mass": {"display": "about 0.95 solar masses", "value": 0.9482}}},
            "planets": [{"name": "A b", "measurements": {"mass": {"display": "minimum-mass quantity (M sin i): a reported lower limit of 2 Earth masses", "provenance": "Msini", "value": 1.991, "error_plus": 0.7}}},
                {"name": "A c", "measurements": {"mass": {"display": "planet mass (not a minimum-mass quantity): a reported upper limit of 5 Earth masses", "provenance": "Mass", "source_field": "pl_bmasse"}}}],
            "source": {"private": "audit"}, "guide": {"minimum_mass": "M sin i"},
            "publishable_comparisons": ["A b's year is shorter than Earth's roughly 365-day year.", "A c's radius is larger than Earth's."], "silent_constraints": ["Private note"], "request": {"target_words": "150-300"}});
        let compact = project(&request);
        assert_eq!(
            compact["planets"][0]["measurements"]["minimum_mass"],
            "a reported lower limit of 2 Earth masses"
        );
        assert_eq!(
            compact["planets"][1]["measurements"]["mass"],
            "a reported upper limit of 5 Earth masses"
        );
        assert_eq!(compact["star"]["measurements"]["mass"], "0.95 solar masses");
        assert_eq!(
            compact["publishable_comparisons"],
            json!(["A c's radius is larger than Earth's."])
        );
        assert_eq!(compact["silent_constraints"], request["silent_constraints"]);
        let wire = compact.to_string();
        for excluded in [
            "source_field",
            "error_plus",
            "1.991",
            "0.9482",
            "catalog_system_planet_count",
            "M sin i",
            "audit",
            "365",
        ] {
            assert!(!wire.contains(excluded), "{excluded}");
        }
    }
}
