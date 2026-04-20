use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use leptos::server_fn::codec::GetUrl;

#[cfg(feature = "ssr")]
use crate::server::data::details;
#[cfg(feature = "ssr")]
use crate::server::handlers::ApiState;
#[cfg(feature = "ssr")]
use leptos::serde_json::Value;
#[cfg(feature = "ssr")]
use std::collections::HashMap;

#[cfg(feature = "ssr")]
use super::ColumnMetadata;
use super::{ExoplanetDetail, HostPlanets, StellarHostDetail};

/// Server function to fetch a single stellar host's details.
#[server(input = GetUrl)]
pub async fn get_stellar_host_detail(
    hostname: String,
) -> Result<StellarHostDetail, ServerFnError> {
    tracing::info!("get_stellar_host_detail called: hostname={hostname}");
    let state = expect_context::<ApiState>();

    let (detail, exo_metadata): (
        StellarHostDetail,
        HashMap<String, exo_core::metadata::ColumnMetadata>,
    ) = details::get_stellar_host_detail_cached(
        &state.stellarhosts_df,
        &state.host_detail_cache,
        &state.stellarhosts_metadata,
        &hostname,
    )
    .await
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

    Ok(StellarHostDetail { metadata, ..detail })
}

/// Server function to fetch planets for a given stellar host.
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
    ) = details::get_planets_by_hostname(
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

/// Server function to fetch all exoplanet records for a planet name.
#[server(input = GetUrl)]
pub async fn get_exoplanet_detail(
    pl_name: String,
) -> Result<ExoplanetDetail, ServerFnError> {
    tracing::info!("get_exoplanet_detail called: pl_name={pl_name}");
    let state = expect_context::<ApiState>();

    let (records, exo_metadata): (
        Vec<Value>,
        HashMap<String, exo_core::metadata::ColumnMetadata>,
    ) = details::get_exoplanet_by_name(
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
