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
    let mut canonical = ExoplanetCanonicalSummary {
        hostname: summarize_stable_field(records, "hostname", metadata)
            .map(into_stable_value_summary),
        discovery_method: summarize_categorical_field(
            records,
            "discoverymethod",
            metadata,
        )
        .map(into_categorical_field_summary),
        discovery_year: summarize_stable_field(records, "disc_year", metadata)
            .map(into_stable_value_summary),
        orbital_period: summarize_numeric_field(records, "pl_orbper", metadata)
            .map(into_numeric_field_summary),
        semi_major_axis: summarize_numeric_field(records, "pl_orbsmax", metadata)
            .map(into_numeric_field_summary),
        radius: summarize_numeric_field(records, "pl_rade", metadata)
            .map(into_numeric_field_summary),
        mass: summarize_numeric_field(records, "pl_bmasse", metadata)
            .map(into_numeric_field_summary),
        density: summarize_numeric_field(records, "pl_dens", metadata)
            .map(into_numeric_field_summary),
        equilibrium_temperature: summarize_numeric_field(
            records, "pl_eqt", metadata,
        )
        .map(into_numeric_field_summary),
    };

    if canonical.mass.is_none() {
        canonical.mass =
            summarize_numeric_field(records, MASS_FALLBACK_KEY, metadata)
                .map(into_numeric_field_summary);
    }

    canonical
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn builds_numeric_median_and_stable_summaries_from_all_records() {
        let records = vec![
            json!({
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
        assert_eq!(radius.value, 1.33);
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
            "hostname": "Test d",
            "pl_masse": 5.0
        })];

        let canonical = build_canonical_exoplanet(&records, &HashMap::new());

        let mass = canonical.mass.as_ref().unwrap();
        assert_eq!(mass.key, "pl_masse");
        assert_eq!(mass.value, 5.0);
        assert!(canonical.radius.is_none());
    }
}
