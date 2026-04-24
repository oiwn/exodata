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
