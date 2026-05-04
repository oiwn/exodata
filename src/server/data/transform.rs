use crate::server::functions::{
    CategoricalFieldSummary, CategoricalValueCount, NumericFieldSummary,
    StableValueSummary,
};

pub fn into_numeric_field_summary(
    summary: super::super::stellarhost_canonical::NumericFieldSummary,
) -> NumericFieldSummary {
    NumericFieldSummary {
        key: summary.key,
        label: summary.label,
        unit: summary.unit,
        value: summary.value,
        measurement_count: summary.measurement_count,
        distinct_count: summary.distinct_count,
        min: summary.min,
        max: summary.max,
        disputed: summary.disputed,
    }
}

pub fn into_stable_value_summary(
    summary: super::super::stellarhost_canonical::StableValueSummary,
) -> StableValueSummary {
    StableValueSummary {
        key: summary.key,
        label: summary.label,
        unit: summary.unit,
        value: summary.value,
        distinct_values: summary.distinct_values,
        disputed: summary.disputed,
    }
}

pub fn into_categorical_field_summary(
    summary: super::super::stellarhost_canonical::CategoricalFieldSummary,
) -> CategoricalFieldSummary {
    CategoricalFieldSummary {
        key: summary.key,
        label: summary.label,
        value: summary.value,
        counts: summary
            .counts
            .into_iter()
            .map(|count| CategoricalValueCount {
                value: count.value,
                count: count.count,
            })
            .collect(),
        disputed: summary.disputed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::stellarhost_canonical;
    use serde_json::json;

    #[test]
    fn converts_numeric_summary_without_losing_fields() {
        let summary = stellarhost_canonical::NumericFieldSummary {
            key: "st_mass".to_string(),
            label: "Mass".to_string(),
            unit: Some("M_sun".to_string()),
            value: 0.91,
            measurement_count: 3,
            distinct_count: 2,
            min: 0.9,
            max: 0.92,
            disputed: true,
        };

        let converted = into_numeric_field_summary(summary);

        assert_eq!(converted.key, "st_mass");
        assert_eq!(converted.label, "Mass");
        assert_eq!(converted.unit.as_deref(), Some("M_sun"));
        assert_eq!(converted.value, 0.91);
        assert_eq!(converted.measurement_count, 3);
        assert_eq!(converted.distinct_count, 2);
        assert_eq!(converted.min, 0.9);
        assert_eq!(converted.max, 0.92);
        assert!(converted.disputed);
    }

    #[test]
    fn converts_stable_summary_without_losing_values() {
        let summary = stellarhost_canonical::StableValueSummary {
            key: "sy_pnum".to_string(),
            label: "Planets".to_string(),
            unit: None,
            value: json!(2),
            distinct_values: vec![json!(1), json!(2)],
            disputed: true,
        };

        let converted = into_stable_value_summary(summary);

        assert_eq!(converted.key, "sy_pnum");
        assert_eq!(converted.label, "Planets");
        assert_eq!(converted.value, json!(2));
        assert_eq!(converted.distinct_values, vec![json!(1), json!(2)]);
        assert!(converted.disputed);
    }

    #[test]
    fn converts_categorical_summary_counts() {
        let summary = stellarhost_canonical::CategoricalFieldSummary {
            key: "st_spectype".to_string(),
            label: "Spectral Type".to_string(),
            value: "G2 V".to_string(),
            counts: vec![
                stellarhost_canonical::CategoricalValueCount {
                    value: "G2 V".to_string(),
                    count: 2,
                },
                stellarhost_canonical::CategoricalValueCount {
                    value: "G3 V".to_string(),
                    count: 1,
                },
            ],
            disputed: true,
        };

        let converted = into_categorical_field_summary(summary);

        assert_eq!(converted.key, "st_spectype");
        assert_eq!(converted.value, "G2 V");
        assert_eq!(converted.counts.len(), 2);
        assert_eq!(converted.counts[0].value, "G2 V");
        assert_eq!(converted.counts[0].count, 2);
        assert!(converted.disputed);
    }
}
