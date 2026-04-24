use super::rows::dataframe_to_json;
use super::transform::{
    into_categorical_field_summary, into_numeric_field_summary,
    into_stable_value_summary,
};
use crate::server::cache::HostDetailCache;
use crate::server::functions::StellarHostDetail;
use crate::server::stellarhost_canonical::build_canonical_host;
use exo_core::metadata::ColumnMetadata;
use polars::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

pub type HostPlanetsResult =
    Result<(Vec<Value>, Vec<String>, HashMap<String, ColumnMetadata>), String>;

pub fn get_stellar_host_by_name(
    df: &DataFrame,
    all_metadata: &HashMap<String, ColumnMetadata>,
    hostname: &str,
) -> Result<(Vec<Value>, HashMap<String, ColumnMetadata>), String> {
    let filtered = df
        .clone()
        .lazy()
        .filter(col("hostname").eq(lit(hostname)))
        .collect()
        .map_err(|e| format!("Failed to filter by hostname: {}", e))?;

    if filtered.height() == 0 {
        return Err(format!("Stellar host '{}' not found", hostname));
    }

    let rows = dataframe_to_json(&filtered)?;
    Ok((rows, all_metadata.clone()))
}

pub async fn get_stellar_host_detail_cached(
    df: &DataFrame,
    host_detail_cache: &HostDetailCache,
    all_metadata: &HashMap<String, ColumnMetadata>,
    hostname: &str,
) -> Result<(StellarHostDetail, HashMap<String, ColumnMetadata>), String> {
    if let Some(cached) = host_detail_cache.get(hostname).await {
        return Ok((cached, all_metadata.clone()));
    }

    let filtered = df
        .clone()
        .lazy()
        .filter(col("hostname").eq(lit(hostname)))
        .collect()
        .map_err(|e| format!("Failed to filter by hostname: {}", e))?;

    if filtered.height() == 0 {
        return Err(format!("Stellar host '{}' not found", hostname));
    }

    let canonical = build_canonical_host(hostname, &filtered, all_metadata)?;
    let detail = StellarHostDetail {
        hostname: canonical.hostname,
        identity: crate::server::functions::HostIdentity {
            hostname: canonical.identity.hostname,
            aliases: canonical.identity.aliases,
        },
        system: crate::server::functions::HostSystemSummary {
            planet_count: canonical
                .system
                .planet_count
                .map(into_stable_value_summary),
            star_count: canonical
                .system
                .star_count
                .map(into_stable_value_summary),
            moon_count: canonical
                .system
                .moon_count
                .map(into_stable_value_summary),
            distance: canonical.system.distance.map(into_numeric_field_summary),
            parallax: canonical.system.parallax.map(into_numeric_field_summary),
        },
        star: crate::server::functions::HostStarSummary {
            spectype: canonical.star.spectype.map(into_categorical_field_summary),
            teff: canonical.star.teff.map(into_numeric_field_summary),
            mass: canonical.star.mass.map(into_numeric_field_summary),
            radius: canonical.star.radius.map(into_numeric_field_summary),
            age: canonical.star.age.map(into_numeric_field_summary),
            luminosity: canonical.star.luminosity.map(into_numeric_field_summary),
            metallicity: canonical
                .star
                .metallicity
                .map(into_numeric_field_summary),
            logg: canonical.star.logg.map(into_numeric_field_summary),
        },
        provenance: crate::server::functions::HostProvenanceSummary {
            record_count: canonical.provenance.record_count,
            stellar_refs: canonical.provenance.stellar_refs,
            system_refs: canonical.provenance.system_refs,
            key_field_stats: canonical
                .provenance
                .key_field_stats
                .into_iter()
                .map(|stat| crate::server::functions::ProvenanceStat {
                    key: stat.key,
                    label: stat.label,
                    measurement_count: stat.measurement_count,
                    distinct_count: stat.distinct_count,
                    disputed: stat.disputed,
                })
                .collect(),
        },
        records: canonical.records,
        provenance_columns: canonical.provenance_columns,
        metadata: HashMap::new(),
    };

    host_detail_cache
        .insert(hostname.to_string(), detail.clone())
        .await;

    Ok((detail, all_metadata.clone()))
}

pub fn get_exoplanet_by_name(
    df: &DataFrame,
    all_metadata: &HashMap<String, ColumnMetadata>,
    pl_name: &str,
) -> Result<(Vec<Value>, HashMap<String, ColumnMetadata>), String> {
    let filtered = df
        .clone()
        .lazy()
        .filter(col("pl_name").eq(lit(pl_name)))
        .collect()
        .map_err(|e| format!("Failed to filter by planet name: {}", e))?;

    if filtered.height() == 0 {
        return Err(format!("Exoplanet '{}' not found", pl_name));
    }

    let rows = dataframe_to_json(&filtered)?;
    Ok((rows, all_metadata.clone()))
}

pub fn get_planets_by_hostname(
    df: &DataFrame,
    all_metadata: &HashMap<String, ColumnMetadata>,
    hostname: &str,
) -> HostPlanetsResult {
    let columns = [
        "pl_name",
        "discoverymethod",
        "disc_year",
        "pl_orbper",
        "pl_rade",
        "pl_bmasse",
        "pl_eqt",
    ];

    let valid_columns: Vec<&str> = columns
        .iter()
        .filter(|c| df.column(c).is_ok())
        .copied()
        .collect();

    let filtered = df
        .clone()
        .lazy()
        .filter(col("hostname").eq(lit(hostname)))
        .select(valid_columns.iter().map(|c| col(*c)).collect::<Vec<_>>())
        .collect()
        .map_err(|e| format!("Failed to filter planets: {}", e))?;

    let filtered = filtered
        .unique::<String, String>(
            Some(&["pl_name".to_string()]),
            UniqueKeepStrategy::First,
            None,
        )
        .map_err(|e| format!("Failed to deduplicate planets: {}", e))?;

    let rows = dataframe_to_json(&filtered)?;
    let column_names: Vec<String> =
        valid_columns.iter().map(|s| s.to_string()).collect();

    let filtered_metadata: HashMap<String, ColumnMetadata> = all_metadata
        .iter()
        .filter(|(k, _)| valid_columns.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    Ok((rows, column_names, filtered_metadata))
}
