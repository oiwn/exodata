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
    if let Some(planets) = host.system.planet_count.as_ref() {
        if let Some(count) = value_as_usize(&planets.value) {
            parts.push(format!("{count} confirmed planets in the system."));
        }
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
