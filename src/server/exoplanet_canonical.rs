use exo_types::metadata::ColumnMetadata;
use serde_json::Value;
use std::collections::HashMap;

use super::data::transform::{
    into_categorical_field_summary, into_numeric_field_summary,
    into_stable_value_summary,
};
use super::functions::ExoplanetCanonicalSummary;
use super::stellarhost_canonical::{
    summarize_categorical_field, summarize_numeric_field, summarize_stable_field,
};

const MASS_FALLBACK_KEY: &str = "pl_masse";

pub fn build_canonical_exoplanet(
    records: &[Value],
    metadata: &HashMap<String, ColumnMetadata>,
) -> ExoplanetCanonicalSummary {
    let selected = exo_core::selection::exoplanet_record_index(records)
        .map(|index| &records[index]);
    let mut canonical = ExoplanetCanonicalSummary {
        hostname: summarize_stable_field(records, "hostname", metadata, selected)
            .map(into_stable_value_summary),
        discovery_method: summarize_categorical_field(
            records,
            "discoverymethod",
            metadata,
            selected,
        )
        .map(into_categorical_field_summary),
        discovery_year: summarize_stable_field(
            records,
            "disc_year",
            metadata,
            selected,
        )
        .map(into_stable_value_summary),
        orbital_period: summarize_numeric_field(
            records,
            "pl_orbper",
            metadata,
            selected,
        )
        .map(into_numeric_field_summary),
        semi_major_axis: summarize_numeric_field(
            records,
            "pl_orbsmax",
            metadata,
            selected,
        )
        .map(into_numeric_field_summary),
        radius: summarize_numeric_field(records, "pl_rade", metadata, selected)
            .map(into_numeric_field_summary),
        mass: summarize_numeric_field(records, "pl_bmasse", metadata, selected)
            .map(into_numeric_field_summary),
        density: summarize_numeric_field(records, "pl_dens", metadata, selected)
            .map(into_numeric_field_summary),
        equilibrium_temperature: summarize_numeric_field(
            records, "pl_eqt", metadata, selected,
        )
        .map(into_numeric_field_summary),
    };

    if canonical.mass.is_none() {
        canonical.mass = summarize_numeric_field(
            records,
            MASS_FALLBACK_KEY,
            metadata,
            selected,
        )
        .map(into_numeric_field_summary);
    }

    canonical
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn uses_default_values_with_all_record_diagnostics() {
        let records = vec![
            json!({
                "default_flag": 1,
                "hostname": "Kepler-10",
                "discoverymethod": "Transit",
                "disc_year": 2011,
                "pl_orbper": 0.837,
                "pl_rade": 1.47,
                "pl_bmasse": 3.33,
                "pl_dens": null,
                "pl_orbsmax": 0.0168,
                "pl_eqt": 2169.0
            }),
            json!({
                "hostname": "Kepler-10",
                "discoverymethod": "Transit",
                "disc_year": 2011,
                "pl_orbper": 0.837,
                "pl_rade": 1.19,
                "pl_bmasse": 2.91,
                "pl_dens": null,
                "pl_orbsmax": 0.0168,
                "pl_eqt": 2169.0
            }),
        ];

        let canonical = build_canonical_exoplanet(&records, &HashMap::new());

        let radius = canonical.radius.as_ref().unwrap();
        assert_eq!(radius.value, 1.47);
        assert_eq!(radius.measurement_count, 2);
        assert_eq!(radius.min, 1.19);
        assert_eq!(radius.max, 1.47);
        assert!(radius.disputed);
        assert_eq!(radius.label, "Radius");

        let hostname = canonical.hostname.as_ref().unwrap();
        assert_eq!(hostname.value, json!("Kepler-10"));
        assert!(!hostname.disputed);

        let discovery = canonical.discovery_method.as_ref().unwrap();
        assert_eq!(discovery.value, "Transit");
        assert_eq!(discovery.counts.len(), 1);
        assert!(!discovery.disputed);

        let year = canonical.discovery_year.as_ref().unwrap();
        assert_eq!(year.value, json!(2011));

        // Missing columns disappear cleanly instead of producing empty cards.
        assert!(canonical.density.is_none());
    }

    #[test]
    fn falls_back_to_pl_masse_when_pl_bmasse_is_absent() {
        let records = vec![json!({
            "default_flag": 1,
            "hostname": "Test d",
            "pl_masse": 5.0
        })];

        let canonical = build_canonical_exoplanet(&records, &HashMap::new());

        let mass = canonical.mass.as_ref().unwrap();
        assert_eq!(mass.key, "pl_masse");
        assert_eq!(mass.value, 5.0);
        assert!(canonical.radius.is_none());
    }

    #[test]
    fn leaves_default_gaps_missing_and_uses_only_same_row_mass_fallback() {
        let rows = vec![
            json!({"default_flag": 0, "pl_rade": 3.0, "pl_bmasse": 40.0, "pl_eqt": 500}),
            json!({"default_flag": 1, "pl_rade": null, "pl_masse": 5.0, "pl_masselim": 1, "pl_refname": "Ref"}),
        ];
        let summary = build_canonical_exoplanet(&rows, &HashMap::new());
        assert!(summary.radius.is_none());
        assert!(summary.equilibrium_temperature.is_none());
        assert_eq!(summary.mass.unwrap().value, 5.0);
        let index = exo_core::selection::exoplanet_record_index(&rows).unwrap();
        assert_eq!(rows[index]["pl_masselim"], 1);
        assert_eq!(rows[index]["pl_refname"], "Ref");
    }

    #[test]
    fn missing_or_ambiguous_defaults_have_no_adopted_measurements() {
        for rows in [
            vec![json!({"pl_rade": 1.0})],
            vec![
                json!({"default_flag": 1, "pl_rade": 1.0}),
                json!({"default_flag": 1, "pl_rade": 3.0}),
            ],
        ] {
            assert!(
                build_canonical_exoplanet(&rows, &HashMap::new())
                    .radius
                    .is_none()
            );
        }
    }
}
