use super::rows::dataframe_to_json;
use super::transform::{
    into_categorical_field_summary, into_numeric_field_summary,
    into_stable_value_summary,
};
use crate::server::cache::HostDetailCache;
use crate::server::functions::StellarHostDetail;
use crate::server::stellarhost_canonical::build_canonical_host;
use exo_types::metadata::ColumnMetadata;
use polars::prelude::*;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

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
        selected_record_index: canonical.selected_record_index,
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
        .collect()
        .map_err(|e| format!("Failed to filter planets: {}", e))?;

    let mut grouped: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in dataframe_to_json(&filtered)? {
        if let Some(name) = row["pl_name"].as_str() {
            grouped.entry(name.to_owned()).or_default().push(row);
        }
    }
    let rows = grouped
        .into_iter()
        .map(|(name, records)| {
            let selected = exo_core::selection::exoplanet_record_index(&records)
                .map(|index| &records[index]);
            let mut row = serde_json::Map::new();
            for &column in &valid_columns {
                let value = if column == "pl_name" {
                    Value::String(name.clone())
                } else {
                    selected
                        .and_then(|record| record.get(column))
                        .cloned()
                        .unwrap_or(Value::Null)
                };
                row.insert(column.to_owned(), value);
            }
            if row.get("pl_bmasse").is_some_and(Value::is_null)
                && let Some(mass) =
                    selected.and_then(|record| record.get("pl_masse"))
            {
                row.insert("pl_bmasse".to_owned(), mass.clone());
            }
            Value::Object(row)
        })
        .collect();
    let column_names: Vec<String> =
        valid_columns.iter().map(|s| s.to_string()).collect();

    let filtered_metadata: HashMap<String, ColumnMetadata> = all_metadata
        .iter()
        .filter(|(k, _)| valid_columns.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    Ok((rows, column_names, filtered_metadata))
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::df;
    use serde_json::json;

    fn metadata_for(names: &[&str]) -> HashMap<String, ColumnMetadata> {
        names
            .iter()
            .map(|name| {
                (
                    (*name).to_string(),
                    ColumnMetadata {
                        name: (*name).to_string(),
                        description: Some(format!("{name} description")),
                        unit: None,
                        datatype: "char".to_string(),
                    },
                )
            })
            .collect()
    }

    #[test]
    fn get_stellar_host_by_name_returns_matching_rows_and_metadata() {
        let df = df! {
            "hostname" => &["Kepler-10", "TRAPPIST-1", "Kepler-10"],
            "sy_dist" => &[173.0, 12.4, 174.0],
        }
        .unwrap();
        let metadata = metadata_for(&["hostname", "sy_dist"]);

        let (rows, returned_metadata) =
            get_stellar_host_by_name(&df, &metadata, "Kepler-10").unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["hostname"], json!("Kepler-10"));
        assert_eq!(returned_metadata.len(), 2);
    }

    #[test]
    fn get_stellar_host_by_name_reports_missing_host() {
        let df = df! {
            "hostname" => &["Kepler-10"],
        }
        .unwrap();

        let error = get_stellar_host_by_name(&df, &HashMap::new(), "Missing")
            .unwrap_err();

        assert!(error.contains("Stellar host 'Missing' not found"));
    }

    #[test]
    fn get_exoplanet_by_name_returns_matching_records() {
        let df = df! {
            "pl_name" => &["Kepler-10 b", "Kepler-10 c"],
            "hostname" => &["Kepler-10", "Kepler-10"],
        }
        .unwrap();

        let (rows, _) =
            get_exoplanet_by_name(&df, &HashMap::new(), "Kepler-10 c").unwrap();

        assert_eq!(
            rows,
            vec![json!({
                "pl_name": "Kepler-10 c",
                "hostname": "Kepler-10"
            })]
        );
    }

    #[test]
    fn get_exoplanet_by_name_reports_missing_planet() {
        let df = df! {
            "pl_name" => &["Kepler-10 b"],
        }
        .unwrap();

        let error =
            get_exoplanet_by_name(&df, &HashMap::new(), "Missing b").unwrap_err();

        assert!(error.contains("Exoplanet 'Missing b' not found"));
    }

    #[test]
    fn get_planets_by_hostname_selects_valid_columns_and_deduplicates() {
        let df = df! {
            "hostname" => &["Kepler-10", "Kepler-10", "TRAPPIST-1"],
            "pl_name" => &["Kepler-10 b", "Kepler-10 b", "TRAPPIST-1 b"],
            "discoverymethod" => &["Transit", "Transit", "Transit"],
            "disc_year" => &[2011_i64, 2011, 2016],
            "pl_rade" => &[9.0, 1.47, 1.12],
            "default_flag" => &[0_i32, 1, 1],
        }
        .unwrap();
        let metadata = metadata_for(&["pl_name", "discoverymethod", "disc_year"]);

        let (rows, columns, filtered_metadata) =
            get_planets_by_hostname(&df, &metadata, "Kepler-10").unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["pl_name"], json!("Kepler-10 b"));
        assert_eq!(rows[0]["pl_rade"], json!(1.47));
        assert_eq!(
            columns,
            vec!["pl_name", "discoverymethod", "disc_year", "pl_rade"]
        );
        assert!(filtered_metadata.contains_key("pl_name"));
        assert!(!filtered_metadata.contains_key("hostname"));
    }

    #[test]
    fn related_planets_preserve_names_without_unique_defaults_and_do_not_fill_gaps()
     {
        let df = df! {
            "hostname" => &["Test", "Test", "Test", "Test", "Test"],
            "pl_name" => &["b", "b", "c", "c", "d"],
            "default_flag" => &[1_i32, 0, 1, 1, 0],
            "pl_rade" => &[None, Some(9.0), Some(1.0), Some(3.0), Some(5.0)],
        }
        .unwrap();
        let (rows, _, _) =
            get_planets_by_hostname(&df, &HashMap::new(), "Test").unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows.iter()
                .map(|r| r["pl_name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["b", "c", "d"]
        );
        assert!(rows.iter().all(|row| row["pl_rade"].is_null()));
    }
}
