//! Compatibility shim for older imports.
//! Pure server data/query logic now lives under `crate::server::data`.
//! TODO: remove this

pub use super::data::details::{
    HostPlanetsResult, get_exoplanet_by_name, get_planets_by_hostname,
    get_stellar_host_by_name, get_stellar_host_detail_cached,
};
pub use super::data::insights::{
    get_distinct_exoplanets_data, get_distinct_stellarhosts_data,
};
pub use super::data::rows::{dataframe_to_json, distinct_value_count};
pub use super::data::tables::{
    TableConfig, TableQuery, TableResult, get_exoplanets_data,
    get_exoplanets_data_cached, get_stellarhosts_data,
    get_stellarhosts_data_cached, get_table_data,
};
pub use super::data::transform::{
    into_categorical_field_summary, into_numeric_field_summary,
    into_stable_value_summary,
};
