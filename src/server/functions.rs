// Leptos server functions for the application
// These functions can be called from the client but execute on the server

use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use crate::server::handlers::ApiState;
use crate::tables::aggregation;

/// Statistics data structure for the overview page
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DataStats {
    pub stellarhosts_total: usize,
    pub exoplanets_total: usize,
    pub avg_stellar_temp: f64,
    pub avg_stellar_distance: f64,
    pub discovery_methods: Vec<(String, usize)>,
    pub planet_size_categories: Vec<(String, usize)>,
}

/// Server function to fetch and calculate overview statistics
#[server(GetStats, "/api")]
pub async fn get_stats() -> Result<DataStats, ServerFnError> {
    // Get ApiState from leptos context
    let state = expect_context::<ApiState>();

    // Get total counts
    let (stellarhosts_total, exoplanets_total) = aggregation::get_total_counts(
        &state.stellarhosts_df,
        &state.exoplanets_df,
    );

    // Get average temperature (default to 0 if None)
    let avg_stellar_temp = aggregation::get_avg_temperature(&state.stellarhosts_df)
        .unwrap_or(0.0);

    // Get average distance (default to 0 if None)
    let avg_stellar_distance = aggregation::get_avg_distance(&state.stellarhosts_df)
        .unwrap_or(0.0);

    // Get top 10 discovery methods
    let discovery_methods = aggregation::get_discovery_methods(&state.exoplanets_df, 10);

    // Get planet size categories
    let planet_size_categories = aggregation::get_planet_size_categories(&state.exoplanets_df);

    Ok(DataStats {
        stellarhosts_total,
        exoplanets_total,
        avg_stellar_temp,
        avg_stellar_distance,
        discovery_methods,
        planet_size_categories,
    })
}
