use leptos::serde_json::Value;
use percent_encoding::{
    NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode,
};

use crate::server::functions::{ExoplanetDetail, StellarHostDetail};

pub const SITE_NAME: &str = "Exodata";
pub const SITE_URL: &str = "https://exodata.space";
pub const DEFAULT_DESCRIPTION: &str = "Search confirmed exoplanets and stellar hosts with server-rendered tables, detail pages, and NASA Exoplanet Archive-based data.";

pub fn title_with_site(title: &str) -> String {
    format!("{title} | {SITE_NAME}")
}

pub fn canonical_url(path: &str) -> String {
    if path.is_empty() || path == "/" {
        format!("{SITE_URL}/")
    } else {
        format!("{SITE_URL}{path}")
    }
}

pub fn encode_path_segment(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

pub fn decode_path_segment(value: &str) -> String {
    percent_decode_str(value).decode_utf8_lossy().into_owned()
}

pub fn overview_title() -> String {
    title_with_site("Exoplanets Catalog")
}

pub fn overview_description() -> String {
    "Explore confirmed exoplanets and their host stars with searchable tables, discovery stats, and server-rendered detail pages.".to_string()
}

pub fn about_title() -> String {
    title_with_site("About the Catalog")
}

pub fn about_description() -> String {
    "Learn how Exodata serves NASA Exoplanet Archive data with Rust, Leptos, Axum, Parquet, and Polars.".to_string()
}

pub fn stellarhosts_title() -> String {
    title_with_site("Stellar Hosts")
}

pub fn stellarhosts_description() -> String {
    "Browse stellar host systems with sortable columns, filters, and detail pages for confirmed exoplanet host stars.".to_string()
}

pub fn exoplanets_title() -> String {
    title_with_site("Exoplanets")
}

pub fn exoplanets_description() -> String {
    "Browse confirmed exoplanets with sortable data, filters, and detail pages for planetary records and host-star context.".to_string()
}

pub fn stellarhost_detail_title(host: &StellarHostDetail) -> String {
    title_with_site(&format!("{} Stellar Host", host.hostname))
}

pub fn stellarhost_detail_description(host: &StellarHostDetail) -> String {
    let mut parts = Vec::new();
    parts.push(format!(
        "Explore the stellar host profile for {}.",
        host.hostname
    ));

    if let Some(teff) = host.star.teff.as_ref() {
        parts.push(format!("Effective temperature {:.0} K.", teff.value));
    }
    if let Some(distance) = host.system.distance.as_ref() {
        parts.push(format!("Distance {:.1} pc.", distance.value));
    }
    if let Some(planets) = host.system.planet_count.as_ref()
        && let Some(count) = value_as_usize(&planets.value)
    {
        parts.push(format!("{count} confirmed planets in the system."));
    }

    parts.join(" ")
}

pub fn exoplanet_detail_title(detail: &ExoplanetDetail) -> String {
    title_with_site(&format!("{} Exoplanet", detail.pl_name))
}

pub fn exoplanet_detail_description(detail: &ExoplanetDetail) -> String {
    let mut parts = Vec::new();
    parts.push(format!(
        "Explore measurements and records for {}.",
        detail.pl_name
    ));

    if let Some(record) = detail.records.first() {
        if let Some(hostname) = record.get("hostname").and_then(|v| v.as_str()) {
            parts.push(format!("Host star: {hostname}."));
        }
        if let Some(method) =
            record.get("discoverymethod").and_then(|v| v.as_str())
        {
            parts.push(format!("Discovery method: {method}."));
        }
        if let Some(year) = value_as_i64(record.get("disc_year")) {
            parts.push(format!("Discovery year: {year}."));
        }
    }

    parts.join(" ")
}

fn value_as_usize(value: &Value) -> Option<usize> {
    value
        .as_u64()
        .and_then(|v| usize::try_from(v).ok())
        .or_else(|| value.as_i64().and_then(|v| usize::try_from(v).ok()))
}

fn value_as_i64(value: Option<&Value>) -> Option<i64> {
    value.and_then(|v| v.as_i64())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::functions::{
        ExoplanetDetail, HostIdentity, HostProvenanceSummary, HostStarSummary,
        HostSystemSummary, NumericFieldSummary, StableValueSummary,
        StellarHostDetail,
    };
    use leptos::serde_json::json;
    use std::collections::HashMap;

    fn host_detail() -> StellarHostDetail {
        StellarHostDetail {
            hostname: "TRAPPIST-1".to_string(),
            identity: HostIdentity {
                hostname: "TRAPPIST-1".to_string(),
                aliases: HashMap::new(),
            },
            system: HostSystemSummary {
                planet_count: Some(StableValueSummary {
                    key: "sy_pnum".to_string(),
                    label: "Planets".to_string(),
                    unit: None,
                    value: json!(7),
                    distinct_values: vec![json!(7)],
                    disputed: false,
                }),
                distance: Some(NumericFieldSummary {
                    key: "sy_dist".to_string(),
                    label: "Distance".to_string(),
                    unit: Some("pc".to_string()),
                    value: 12.43,
                    measurement_count: 2,
                    distinct_count: 1,
                    min: 12.4,
                    max: 12.5,
                    disputed: false,
                }),
                ..Default::default()
            },
            star: HostStarSummary {
                teff: Some(NumericFieldSummary {
                    key: "st_teff".to_string(),
                    label: "Temperature".to_string(),
                    unit: Some("K".to_string()),
                    value: 2566.0,
                    measurement_count: 1,
                    distinct_count: 1,
                    min: 2566.0,
                    max: 2566.0,
                    disputed: false,
                }),
                ..Default::default()
            },
            provenance: HostProvenanceSummary {
                record_count: 1,
                stellar_refs: vec![],
                system_refs: vec![],
                key_field_stats: vec![],
            },
            records: vec![],
            provenance_columns: vec![],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn canonical_url_handles_root_empty_and_nested_paths() {
        assert_eq!(canonical_url(""), "https://exodata.space/");
        assert_eq!(canonical_url("/"), "https://exodata.space/");
        assert_eq!(
            canonical_url("/stellarhosts/TRAPPIST-1"),
            "https://exodata.space/stellarhosts/TRAPPIST-1"
        );
    }

    #[test]
    fn path_segments_round_trip_reserved_characters() {
        let encoded = encode_path_segment("Kepler-10 b/alpha");

        assert_eq!(encoded, "Kepler%2D10%20b%2Falpha");
        assert_eq!(decode_path_segment(&encoded), "Kepler-10 b/alpha");
    }

    #[test]
    fn stellarhost_detail_description_includes_available_facts() {
        let description = stellarhost_detail_description(&host_detail());

        assert!(description.contains("TRAPPIST-1"));
        assert!(description.contains("Effective temperature 2566 K."));
        assert!(description.contains("Distance 12.4 pc."));
        assert!(description.contains("7 confirmed planets"));
    }

    #[test]
    fn exoplanet_detail_description_uses_first_record() {
        let detail = ExoplanetDetail {
            pl_name: "Kepler-10 b".to_string(),
            canonical: Default::default(),
            records: vec![json!({
                "hostname": "Kepler-10",
                "discoverymethod": "Transit",
                "disc_year": 2011
            })],
            metadata: HashMap::new(),
        };

        let description = exoplanet_detail_description(&detail);

        assert!(description.contains("Kepler-10 b"));
        assert!(description.contains("Host star: Kepler-10."));
        assert!(description.contains("Discovery method: Transit."));
        assert!(description.contains("Discovery year: 2011."));
    }

    #[test]
    fn titles_include_site_suffix() {
        assert_eq!(title_with_site("Docs"), "Docs | Exodata");
        assert_eq!(overview_title(), "Exoplanets Catalog | Exodata");
        assert_eq!(about_title(), "About the Catalog | Exodata");
        assert_eq!(stellarhosts_title(), "Stellar Hosts | Exodata");
        assert_eq!(exoplanets_title(), "Exoplanets | Exodata");
    }
}
