// Leptos server functions for the application.
// These functions can be called from the client but execute on the server.

use exo_types::metadata::ColumnMetadata;
use leptos::prelude::*;
use leptos::serde_json::Value;
use leptos::server_fn::ServerFnError;
use leptos::server_fn::codec::GetUrl;
use std::collections::HashMap;

#[cfg(feature = "ssr")]
use crate::server::handlers::ApiState;

pub mod details;
pub mod insights;
pub mod tables;

pub use details::{
    get_exoplanet_detail, get_planets_for_host, get_stellar_host_detail,
};
pub use insights::get_insight;
pub use tables::{get_exoplanets_page, get_stellarhosts_page};

/// Statistics data structure for the overview page.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DataStats {
    pub stellarhosts_total: usize,
    pub exoplanets_total: usize,
    pub avg_stellar_temp: f64,
    pub avg_stellar_distance: f64,
    pub discovery_methods: Vec<(String, usize)>,
    pub planet_size_categories: Vec<(String, usize)>,
    pub planet_mass_bands: Vec<(String, usize)>,
    pub stellar_classes: Vec<(String, usize)>,
    pub planet_temperature_bands: Vec<(String, usize)>,
    pub detection_sources: Vec<(String, usize)>,
    pub discovery_years: Vec<(String, usize)>,
    pub orbital_period_buckets: Vec<(String, usize)>,
}

/// Server function to fetch precomputed overview statistics.
#[server(input = GetUrl)]
pub async fn get_stats() -> Result<DataStats, ServerFnError> {
    let state = expect_context::<ApiState>();
    Ok(state.overview_stats.as_ref().clone())
}

/// Table data structure for paginated table-like responses.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct TableData {
    pub rows: Vec<Value>,
    pub columns: Vec<String>,
    pub total: usize,
    pub total_all: usize,
    pub page: usize,
    pub limit: usize,
}

/// Stellar host detail data structure.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct StellarHostDetail {
    pub hostname: String,
    pub identity: HostIdentity,
    pub system: HostSystemSummary,
    pub star: HostStarSummary,
    pub provenance: HostProvenanceSummary,
    pub records: Vec<Value>,
    pub provenance_columns: Vec<String>,
    pub metadata: HashMap<String, ColumnMetadata>,
}

/// Planets for a stellar host.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct HostPlanets {
    pub hostname: String,
    pub planets: Vec<Value>,
    pub columns: Vec<String>,
    pub metadata: HashMap<String, ColumnMetadata>,
}

/// Exoplanet detail data structure.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ExoplanetDetail {
    pub pl_name: String,
    pub canonical: ExoplanetCanonicalSummary,
    pub records: Vec<Value>,
    pub metadata: HashMap<String, ColumnMetadata>,
}

/// Canonical adopted planet values computed from all detail records.
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct ExoplanetCanonicalSummary {
    pub hostname: Option<StableValueSummary>,
    pub discovery_method: Option<CategoricalFieldSummary>,
    pub discovery_year: Option<StableValueSummary>,
    pub orbital_period: Option<NumericFieldSummary>,
    pub semi_major_axis: Option<NumericFieldSummary>,
    pub radius: Option<NumericFieldSummary>,
    pub mass: Option<NumericFieldSummary>,
    pub density: Option<NumericFieldSummary>,
    pub equilibrium_temperature: Option<NumericFieldSummary>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct HostIdentity {
    pub hostname: String,
    pub aliases: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct HostSystemSummary {
    pub planet_count: Option<StableValueSummary>,
    pub star_count: Option<StableValueSummary>,
    pub moon_count: Option<StableValueSummary>,
    pub distance: Option<NumericFieldSummary>,
    pub parallax: Option<NumericFieldSummary>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
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

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
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

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct StableValueSummary {
    pub key: String,
    pub label: String,
    pub unit: Option<String>,
    pub value: Value,
    pub distinct_values: Vec<Value>,
    pub disputed: bool,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CategoricalValueCount {
    pub value: String,
    pub count: usize,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CategoricalFieldSummary {
    pub key: String,
    pub label: String,
    pub value: String,
    pub counts: Vec<CategoricalValueCount>,
    pub disputed: bool,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ProvenanceStat {
    pub key: String,
    pub label: String,
    pub measurement_count: usize,
    pub distinct_count: usize,
    pub disputed: bool,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct HostProvenanceSummary {
    pub record_count: usize,
    pub stellar_refs: Vec<String>,
    pub system_refs: Vec<String>,
    pub key_field_stats: Vec<ProvenanceStat>,
}
