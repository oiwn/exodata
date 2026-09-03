use exo_types::metadata::ColumnMetadata;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

const PROVENANCE_COLUMNS: &[&str] = &[
    "st_teff",
    "st_mass",
    "st_rad",
    "st_age",
    "st_lum",
    "st_spectype",
    "sy_dist",
    "st_refname",
    "sy_refname",
];

const NUMERIC_SUMMARY_FIELDS: &[&str] = &[
    "sy_dist", "sy_plx", "st_teff", "st_mass", "st_rad", "st_age", "st_lum",
    "st_logg", "st_met",
];

const STABLE_SYSTEM_FIELDS: &[&str] = &["sy_pnum", "sy_snum", "sy_mnum"];
const ALIAS_FIELDS: &[&str] = &["hd_name", "hip_name", "tic_id"];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanonicalStellarHost {
    pub hostname: String,
    pub identity: HostIdentity,
    pub system: HostSystemSummary,
    pub star: HostStarSummary,
    pub provenance: HostProvenanceSummary,
    pub records: Vec<Value>,
    pub provenance_columns: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostIdentity {
    pub hostname: String,
    pub aliases: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HostSystemSummary {
    pub planet_count: Option<StableValueSummary>,
    pub star_count: Option<StableValueSummary>,
    pub moon_count: Option<StableValueSummary>,
    pub distance: Option<NumericFieldSummary>,
    pub parallax: Option<NumericFieldSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HostStarSummary {
    pub spectype: Option<CategoricalFieldSummary>,
    pub teff: Option<NumericFieldSummary>,
    pub mass: Option<NumericFieldSummary>,
    pub radius: Option<NumericFieldSummary>,
    pub age: Option<NumericFieldSummary>,
    pub luminosity: Option<NumericFieldSummary>,
    pub metallicity: Option<NumericFieldSummary>,
    pub logg: Option<NumericFieldSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NumericFieldSummary {
    pub key: String,
    pub label: String,
    pub unit: Option<String>,
    pub value: f64,
    pub measurement_count: usize,
    pub distinct_count: usize,
    pub min: f64,
    pub max: f64,
    pub disputed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StableValueSummary {
    pub key: String,
    pub label: String,
    pub unit: Option<String>,
    pub value: Value,
    pub distinct_values: Vec<Value>,
    pub disputed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoricalValueCount {
    pub value: String,
    pub count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoricalFieldSummary {
    pub key: String,
    pub label: String,
    pub value: String,
    pub counts: Vec<CategoricalValueCount>,
    pub disputed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvenanceStat {
    pub key: String,
    pub label: String,
    pub measurement_count: usize,
    pub distinct_count: usize,
    pub disputed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostProvenanceSummary {
    pub record_count: usize,
    pub stellar_refs: Vec<String>,
    pub system_refs: Vec<String>,
    pub key_field_stats: Vec<ProvenanceStat>,
}

pub fn build_canonical_host(
    hostname: &str,
    df: &DataFrame,
    all_metadata: &HashMap<String, ColumnMetadata>,
) -> Result<CanonicalStellarHost, String> {
    let records = dataframe_to_json(df)?;
    let identity = build_identity(hostname, &records);
    let system = build_system_summary(&records, all_metadata);
    let star = build_star_summary(&records, all_metadata);
    let provenance = build_provenance_summary(&records);
    let provenance_columns = PROVENANCE_COLUMNS
        .iter()
        .filter(|key| df.column(key).is_ok())
        .map(|key| (*key).to_string())
        .collect();

    Ok(CanonicalStellarHost {
        hostname: hostname.to_string(),
        identity,
        system,
        star,
        provenance,
        records,
        provenance_columns,
    })
}

pub fn build_provenance_records(df: &DataFrame) -> Result<Vec<Value>, String> {
    let valid_columns: Vec<&str> = PROVENANCE_COLUMNS
        .iter()
        .filter(|column| df.column(column).is_ok())
        .copied()
        .collect();

    let selection = if valid_columns.is_empty() {
        df.clone()
    } else {
        df.select(valid_columns)
            .map_err(|e| format!("Failed to select provenance columns: {e}"))?
    };

    dataframe_to_json(&selection)
}

fn build_identity(hostname: &str, records: &[Value]) -> HostIdentity {
    let aliases = ALIAS_FIELDS
        .iter()
        .map(|field| {
            let mut values: Vec<String> =
                collect_strings(records, field).into_iter().collect();
            values.sort();
            ((*field).to_string(), values)
        })
        .collect();

    HostIdentity {
        hostname: hostname.to_string(),
        aliases,
    }
}

fn build_system_summary(
    records: &[Value],
    metadata: &HashMap<String, ColumnMetadata>,
) -> HostSystemSummary {
    let stable_map: HashMap<&str, StableValueSummary> = STABLE_SYSTEM_FIELDS
        .iter()
        .filter_map(|key| {
            summarize_stable_field(records, key, metadata)
                .map(|summary| (*key, summary))
        })
        .collect();

    HostSystemSummary {
        planet_count: stable_map.get("sy_pnum").cloned(),
        star_count: stable_map.get("sy_snum").cloned(),
        moon_count: stable_map.get("sy_mnum").cloned(),
        distance: summarize_numeric_field(records, "sy_dist", metadata),
        parallax: summarize_numeric_field(records, "sy_plx", metadata),
    }
}

fn build_star_summary(
    records: &[Value],
    metadata: &HashMap<String, ColumnMetadata>,
) -> HostStarSummary {
    HostStarSummary {
        spectype: summarize_categorical_field(records, "st_spectype", metadata),
        teff: summarize_numeric_field(records, "st_teff", metadata),
        mass: summarize_numeric_field(records, "st_mass", metadata),
        radius: summarize_numeric_field(records, "st_rad", metadata),
        age: summarize_numeric_field(records, "st_age", metadata),
        luminosity: summarize_numeric_field(records, "st_lum", metadata),
        metallicity: summarize_numeric_field(records, "st_met", metadata),
        logg: summarize_numeric_field(records, "st_logg", metadata),
    }
}

fn build_provenance_summary(records: &[Value]) -> HostProvenanceSummary {
    let mut stellar_refs: Vec<String> =
        collect_strings(records, "st_refname").into_iter().collect();
    stellar_refs.sort();

    let mut system_refs: Vec<String> =
        collect_strings(records, "sy_refname").into_iter().collect();
    system_refs.sort();

    let key_field_stats = NUMERIC_SUMMARY_FIELDS
        .iter()
        .filter_map(|key| {
            let values = collect_f64_values(records, key);
            if values.is_empty() {
                return None;
            }

            Some(ProvenanceStat {
                key: (*key).to_string(),
                label: humanize_key(key),
                measurement_count: values.len(),
                distinct_count: count_distinct_f64(&values),
                disputed: count_distinct_f64(&values) > 1,
            })
        })
        .chain(summarize_categorical_provenance(records, "st_spectype"))
        .collect();

    HostProvenanceSummary {
        record_count: records.len(),
        stellar_refs,
        system_refs,
        key_field_stats,
    }
}

fn summarize_categorical_provenance<'a>(
    records: &'a [Value],
    key: &'a str,
) -> impl Iterator<Item = ProvenanceStat> + 'a {
    let counts = collect_string_counts(records, key);
    let total: usize = counts.iter().map(|(_, count)| *count).sum();
    let distinct_count = counts.len();

    std::iter::once_with(move || ProvenanceStat {
        key: key.to_string(),
        label: humanize_key(key),
        measurement_count: total,
        distinct_count,
        disputed: distinct_count > 1,
    })
    .filter(move |_| total > 0)
}

pub(crate) fn summarize_numeric_field(
    records: &[Value],
    key: &str,
    metadata: &HashMap<String, ColumnMetadata>,
) -> Option<NumericFieldSummary> {
    let mut values = collect_f64_values(records, key);
    if values.is_empty() {
        return None;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let min = values[0];
    let max = *values.last().unwrap_or(&min);
    let distinct_count = count_distinct_f64(&values);

    Some(NumericFieldSummary {
        key: key.to_string(),
        label: humanize_key(key),
        unit: metadata.get(key).and_then(|m| m.unit.clone()),
        value: median(&values),
        measurement_count: values.len(),
        distinct_count,
        min,
        max,
        disputed: distinct_count > 1,
    })
}

pub(crate) fn summarize_stable_field(
    records: &[Value],
    key: &str,
    metadata: &HashMap<String, ColumnMetadata>,
) -> Option<StableValueSummary> {
    let mut distinct = distinct_non_null_values(records, key);
    if distinct.is_empty() {
        return None;
    }

    distinct.sort_by(stable_value_cmp);
    let value = distinct[0].clone();

    Some(StableValueSummary {
        key: key.to_string(),
        label: humanize_key(key),
        unit: metadata.get(key).and_then(|m| m.unit.clone()),
        value,
        distinct_values: distinct.clone(),
        disputed: distinct.len() > 1,
    })
}

pub(crate) fn summarize_categorical_field(
    records: &[Value],
    key: &str,
    _metadata: &HashMap<String, ColumnMetadata>,
) -> Option<CategoricalFieldSummary> {
    let mut counts = collect_string_counts(records, key);
    if counts.is_empty() {
        return None;
    }

    counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let value = counts[0].0.clone();
    let disputed = counts.len() > 1;
    let counts = counts
        .into_iter()
        .map(|(value, count)| CategoricalValueCount { value, count })
        .collect();

    Some(CategoricalFieldSummary {
        key: key.to_string(),
        label: humanize_key(key),
        value,
        counts,
        disputed,
    })
}

fn collect_strings(records: &[Value], key: &str) -> HashSet<String> {
    records
        .iter()
        .filter_map(|record| record.get(key))
        .filter_map(|value| value.as_str())
        .map(ToString::to_string)
        .filter(|value| !value.trim().is_empty())
        .collect()
}

fn collect_string_counts(records: &[Value], key: &str) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for value in records
        .iter()
        .filter_map(|record| record.get(key))
        .filter_map(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }

    counts.into_iter().collect()
}

fn collect_f64_values(records: &[Value], key: &str) -> Vec<f64> {
    records
        .iter()
        .filter_map(|record| record.get(key))
        .filter_map(value_as_f64)
        .filter(|value| value.is_finite())
        .collect()
}

fn distinct_non_null_values(records: &[Value], key: &str) -> Vec<Value> {
    let mut seen = HashSet::new();
    let mut values = Vec::new();

    for value in records
        .iter()
        .filter_map(|record| record.get(key))
        .filter(|value| !value.is_null())
    {
        let normalized = serde_json::to_string(value).unwrap_or_default();
        if seen.insert(normalized) {
            values.push(value.clone());
        }
    }

    values
}

fn stable_value_cmp(left: &Value, right: &Value) -> Ordering {
    match (value_as_f64(left), value_as_f64(right)) {
        (Some(a), Some(b)) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
        _ => left.to_string().cmp(&right.to_string()),
    }
}

fn count_distinct_f64(values: &[f64]) -> usize {
    let mut seen = HashSet::new();
    for value in values {
        seen.insert(format!("{value:.12}"));
    }
    seen.len()
}

fn median(values: &[f64]) -> f64 {
    let len = values.len();
    if len == 0 {
        return 0.0;
    }

    let middle = len / 2;
    if len % 2 == 1 {
        values[middle]
    } else {
        (values[middle - 1] + values[middle]) / 2.0
    }
}

fn humanize_key(key: &str) -> String {
    match key {
        "sy_dist" => "Distance".to_string(),
        "sy_plx" => "Parallax".to_string(),
        "st_teff" => "Temperature".to_string(),
        "st_mass" => "Mass".to_string(),
        "st_rad" => "Radius".to_string(),
        "st_age" => "Age".to_string(),
        "st_lum" => "Luminosity".to_string(),
        "st_logg" => "Surface Gravity".to_string(),
        "st_met" => "Metallicity".to_string(),
        "st_spectype" => "Spectral Type".to_string(),
        "sy_pnum" => "Planets".to_string(),
        "sy_snum" => "Stars".to_string(),
        "sy_mnum" => "Moons".to_string(),
        "hostname" => "Host Star".to_string(),
        "pl_rade" => "Radius".to_string(),
        "pl_bmasse" => "Mass".to_string(),
        "pl_masse" => "Mass".to_string(),
        "pl_dens" => "Density".to_string(),
        "pl_orbper" => "Orbital Period".to_string(),
        "pl_orbsmax" => "Semi-Major Axis".to_string(),
        "pl_eqt" => "Equilibrium Temperature".to_string(),
        "discoverymethod" => "Discovery Method".to_string(),
        "disc_year" => "Discovery Year".to_string(),
        _ => key.to_string(),
    }
}

fn value_as_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn dataframe_to_json(df: &DataFrame) -> Result<Vec<Value>, String> {
    let mut rows = Vec::new();
    let columns = df.get_column_names();

    for row_idx in 0..df.height() {
        let mut row_map = Map::new();

        for col_name in &columns {
            if let Ok(col) = df.column(col_name) {
                let value = match col.dtype() {
                    DataType::String => col
                        .str()
                        .map_err(|e| format!("Failed to get string column: {e}"))?
                        .get(row_idx)
                        .map(|s| json!(s))
                        .unwrap_or(json!(null)),
                    DataType::Float64 => col
                        .f64()
                        .map_err(|e| format!("Failed to get f64 column: {e}"))?
                        .get(row_idx)
                        .map(|f| json!(f))
                        .unwrap_or(json!(null)),
                    DataType::Float32 => col
                        .f32()
                        .map_err(|e| format!("Failed to get f32 column: {e}"))?
                        .get(row_idx)
                        .map(|f| json!(f))
                        .unwrap_or(json!(null)),
                    DataType::Int64 => col
                        .i64()
                        .map_err(|e| format!("Failed to get i64 column: {e}"))?
                        .get(row_idx)
                        .map(|i| json!(i))
                        .unwrap_or(json!(null)),
                    DataType::Int32 => col
                        .i32()
                        .map_err(|e| format!("Failed to get i32 column: {e}"))?
                        .get(row_idx)
                        .map(|i| json!(i))
                        .unwrap_or(json!(null)),
                    DataType::UInt32 => col
                        .u32()
                        .map_err(|e| format!("Failed to get u32 column: {e}"))?
                        .get(row_idx)
                        .map(|i| json!(i))
                        .unwrap_or(json!(null)),
                    DataType::UInt64 => col
                        .u64()
                        .map_err(|e| format!("Failed to get u64 column: {e}"))?
                        .get(row_idx)
                        .map(|i| json!(i))
                        .unwrap_or(json!(null)),
                    _ => json!(null),
                };
                row_map.insert(col_name.to_string(), value);
            }
        }

        rows.push(json!(row_map));
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::df;

    #[test]
    fn builds_canonical_numeric_and_categorical_summaries() {
        let df = df! {
            "hostname" => &["HD 189733", "HD 189733", "HD 189733"],
            "st_teff" => &[4875.0, 5050.0, 5000.0],
            "st_mass" => &[0.78, 0.82, 0.81],
            "st_spectype" => &["K1-K2 V", "K1-K2 V", "K2 V"],
            "sy_dist" => &[19.8, 19.3, 19.3],
            "sy_pnum" => &[1i64, 1i64, 1i64],
            "st_refname" => &["Ref A", "Ref B", "Ref A"],
            "sy_refname" => &["Sys A", "Sys B", "Sys A"],
        }
        .unwrap();

        let host =
            build_canonical_host("HD 189733", &df, &HashMap::new()).unwrap();

        assert_eq!(host.hostname, "HD 189733");
        assert_eq!(host.provenance.record_count, 3);
        assert_eq!(host.star.teff.as_ref().unwrap().value, 5000.0);
        assert!(host.star.teff.as_ref().unwrap().disputed);
        assert_eq!(host.star.spectype.as_ref().unwrap().value, "K1-K2 V");
        assert_eq!(host.system.planet_count.as_ref().unwrap().value, json!(1));
        assert_eq!(host.provenance.stellar_refs.len(), 2);
    }
}
