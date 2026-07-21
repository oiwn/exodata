use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use leptos::server_fn::codec::GetUrl;

use super::{ExoplanetDetail, HostPlanets, StellarHostDetail};
#[cfg(feature = "ssr")]
use crate::server::data::details;
#[cfg(feature = "ssr")]
use crate::server::handlers::ApiState;

/// Server function to fetch a single stellar host's details.
#[server(input = GetUrl)]
pub async fn get_stellar_host_detail(
    hostname: String,
) -> Result<StellarHostDetail, ServerFnError> {
    tracing::info!("get_stellar_host_detail called: hostname={hostname}");
    let state = expect_context::<ApiState>();

    let (detail, metadata) = details::get_stellar_host_detail_cached(
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

    Ok(StellarHostDetail { metadata, ..detail })
}

/// Server function to fetch planets for a given stellar host.
#[server(input = GetUrl)]
pub async fn get_planets_for_host(
    hostname: String,
) -> Result<HostPlanets, ServerFnError> {
    tracing::info!("get_planets_for_host called: hostname={hostname}");
    let state = expect_context::<ApiState>();

    let (planets, columns, metadata) = details::get_planets_by_hostname(
        &state.exoplanets_df,
        &state.exoplanets_metadata,
        &hostname,
    )
    .map_err(|e: String| -> ServerFnError {
        tracing::error!("get_planets_for_host error: {e}");
        ServerFnError::ServerError(e)
    })?;

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

    let (records, metadata) = details::get_exoplanet_by_name(
        &state.exoplanets_df,
        &state.exoplanets_metadata,
        &pl_name,
    )
    .map_err(|e: String| -> ServerFnError {
        tracing::error!("get_exoplanet_detail error: {e}");
        ServerFnError::ServerError(e)
    })?;

    Ok(ExoplanetDetail {
        pl_name,
        records,
        metadata,
    })
}
