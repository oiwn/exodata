// Leptos server functions for the application
// These functions can be called from the client but execute on the server

use leptos::prelude::*;
use leptos::serde_json::Value;
use leptos::server_fn::ServerFnError; // Needed for TableData on both client and server
use leptos::server_fn::codec::GetUrl; // For GET request encoding
use std::collections::HashMap;

// Server-only imports
#[cfg(feature = "ssr")]
use crate::server::common;
#[cfg(feature = "ssr")]
use crate::server::handlers::ApiState;

// NOTE: This is a temporary duplicate of exo_core::metadata::ColumnMetadata
// to avoid bringing exo-core dependencies into the client WASM bundle.
// This will be resolved in the future by restructuring exo-core with feature flags
// or extracting shared types into a separate lightweight crate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub datatype: String,
}

// Helper to convert from exo-core's ColumnMetadata to our local copy
#[cfg(feature = "ssr")]
impl From<exo_core::metadata::ColumnMetadata> for ColumnMetadata {
    fn from(meta: exo_core::metadata::ColumnMetadata) -> Self {
        ColumnMetadata {
            name: meta.name,
            description: meta.description,
            unit: meta.unit,
            datatype: meta.datatype,
        }
    }
}

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
#[server(input = GetUrl)]
pub async fn get_stats() -> Result<DataStats, ServerFnError> {
    // Get ApiState from leptos context
    let state = expect_context::<ApiState>();
    Ok(state.overview_stats.as_ref().clone())
}

/// Table data structure for paginated tables
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct TableData {
    pub rows: Vec<Value>,
    pub columns: Vec<String>,
    pub total: usize,     // Filtered count (for pagination)
    pub total_all: usize, // Unfiltered count (entire dataset)
    pub page: usize,
    pub limit: usize,
}

/// Stellar host detail data structure
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct StellarHostDetail {
    pub hostname: String,
    pub records: Vec<Value>,
    pub metadata: HashMap<String, ColumnMetadata>,
}

/// Planets for a stellar host
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct HostPlanets {
    pub hostname: String,
    pub planets: Vec<Value>,
    pub columns: Vec<String>,
    pub metadata: HashMap<String, ColumnMetadata>,
}

/// Exoplanet detail data structure (all rows for a planet)
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ExoplanetDetail {
    pub pl_name: String,
    pub records: Vec<Value>,
    pub metadata: HashMap<String, ColumnMetadata>,
}

/// Server function to fetch paginated stellar hosts data
///
/// This is a thin wrapper around the common business logic.
/// It handles Leptos-specific concerns (context extraction, error mapping)
/// and delegates the actual work to common::get_stellarhosts_data.
#[server(input = GetUrl)]
pub async fn get_stellarhosts_page(
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    columns: Option<String>,
    filter: Option<String>,
) -> Result<TableData, ServerFnError> {
    tracing::info!("get_stellarhosts_page called: page={page} columns={columns:?}");
    let state = expect_context::<ApiState>();

    // Parse columns parameter (comma-separated column names)
    let selected_columns = columns.map(|s| {
        s.split(',')
            .map(|col| col.trim().to_string())
            .collect::<Vec<_>>()
    });

    // Call the common business logic
    let (rows, total, total_all, columns) = common::get_stellarhosts_data_cached(
        &state.stellarhosts_df,
        &state.table_cache,
        common::TableQuery {
            page,
            limit,
            sort_by,
            order,
            selected_columns,
            filter,
        },
    )
    .await
    .map_err(|e: String| -> ServerFnError {
        tracing::error!("get_stellarhosts_page error: {e}");
        ServerFnError::ServerError(e)
    })?;

    tracing::info!("get_stellarhosts_page done: total={total}");
    Ok(TableData {
        rows,
        columns,
        total,
        total_all,
        page,
        limit,
    })
}

/// Server function to fetch paginated exoplanets data
///
/// This is a thin wrapper around the common business logic.
/// It handles Leptos-specific concerns (context extraction, error mapping)
/// and delegates the actual work to common::get_exoplanets_data.
#[server(input = GetUrl)]
pub async fn get_exoplanets_page(
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    columns: Option<String>,
    filter: Option<String>,
) -> Result<TableData, ServerFnError> {
    let state = expect_context::<ApiState>();
    tracing::info!("get_exoplanets_page called: page={page} columns={columns:?}");

    // Parse columns parameter (comma-separated column names)
    let selected_columns = columns.map(|s| {
        s.split(',')
            .map(|col| col.trim().to_string())
            .collect::<Vec<_>>()
    });

    // Call the common business logic
    let (rows, total, total_all, columns) = common::get_exoplanets_data_cached(
        &state.exoplanets_df,
        &state.table_cache,
        common::TableQuery {
            page,
            limit,
            sort_by,
            order,
            selected_columns,
            filter,
        },
    )
    .await
    .map_err(|e: String| -> ServerFnError {
        tracing::error!("get_exoplanets_page error: {e}");
        ServerFnError::ServerError(e)
    })?;

    tracing::info!("get_exoplanets_page done: total={total}");
    // Wrap in TableData response
    Ok(TableData {
        rows,
        columns,
        total,
        total_all,
        page,
        limit,
    })
}

/// Server function to fetch a single stellar host's details
#[server(input = GetUrl)]
pub async fn get_stellar_host_detail(
    hostname: String,
) -> Result<StellarHostDetail, ServerFnError> {
    tracing::info!("get_stellar_host_detail called: hostname={hostname}");
    let state = expect_context::<ApiState>();

    let (records, exo_metadata): (
        Vec<Value>,
        HashMap<String, exo_core::metadata::ColumnMetadata>,
    ) = common::get_stellar_host_by_name(
        &state.stellarhosts_df,
        &state.stellarhosts_metadata,
        &hostname,
    )
    .map_err(|e: String| -> ServerFnError {
        tracing::error!("get_stellar_host_detail error: {e}");
        ServerFnError::ServerError(e)
    })?;

    let metadata: HashMap<String, ColumnMetadata> = exo_metadata
        .into_iter()
        .map(|(key, val): (String, exo_core::metadata::ColumnMetadata)| {
            (key, val.into())
        })
        .collect();

    Ok(StellarHostDetail {
        hostname,
        records,
        metadata,
    })
}

/// Server function to fetch planets for a given stellar host
#[server(input = GetUrl)]
pub async fn get_planets_for_host(
    hostname: String,
) -> Result<HostPlanets, ServerFnError> {
    tracing::info!("get_planets_for_host called: hostname={hostname}");
    let state = expect_context::<ApiState>();

    let (planets, columns, exo_metadata): (
        Vec<Value>,
        Vec<String>,
        HashMap<String, exo_core::metadata::ColumnMetadata>,
    ) = common::get_planets_by_hostname(
        &state.exoplanets_df,
        &state.exoplanets_metadata,
        &hostname,
    )
    .map_err(|e: String| -> ServerFnError {
        tracing::error!("get_planets_for_host error: {e}");
        ServerFnError::ServerError(e)
    })?;

    let metadata: HashMap<String, ColumnMetadata> = exo_metadata
        .into_iter()
        .map(|(key, val): (String, exo_core::metadata::ColumnMetadata)| {
            (key, val.into())
        })
        .collect();

    Ok(HostPlanets {
        hostname,
        planets,
        columns,
        metadata,
    })
}

/// Server function to fetch all exoplanet records for a planet name
#[server(input = GetUrl)]
pub async fn get_exoplanet_detail(
    pl_name: String,
) -> Result<ExoplanetDetail, ServerFnError> {
    tracing::info!("get_exoplanet_detail called: pl_name={pl_name}");
    let state = expect_context::<ApiState>();

    let (records, exo_metadata): (
        Vec<Value>,
        HashMap<String, exo_core::metadata::ColumnMetadata>,
    ) = common::get_exoplanet_by_name(
        &state.exoplanets_df,
        &state.exoplanets_metadata,
        &pl_name,
    )
    .map_err(|e: String| -> ServerFnError {
        tracing::error!("get_exoplanet_detail error: {e}");
        ServerFnError::ServerError(e)
    })?;

    let metadata: HashMap<String, ColumnMetadata> = exo_metadata
        .into_iter()
        .map(|(key, val): (String, exo_core::metadata::ColumnMetadata)| {
            (key, val.into())
        })
        .collect();

    Ok(ExoplanetDetail {
        pl_name,
        records,
        metadata,
    })
}
