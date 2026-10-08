use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use leptos::server_fn::codec::GetUrl;

use super::{ExoplanetDetail, HostPlanets, StellarHostDetail};
#[cfg(feature = "ssr")]
use crate::server::data::details;
#[cfg(feature = "ssr")]
use crate::server::exoplanet_canonical;
#[cfg(feature = "ssr")]
use crate::server::handlers::ApiState;

/// Server function to fetch a generated prose description for a stellar
/// host, when one is stored under the content directory. `None` means no
/// description exists for this host.
#[server(input = GetUrl)]
pub async fn get_host_description(
    hostname: String,
) -> Result<Option<String>, ServerFnError> {
    let state = expect_context::<ApiState>();
    Ok(exo_core::selection::system_identifier(&hostname)
        .and_then(|id| state.host_descriptions.get(&id).cloned()))
}

/// Server function to fetch a single stellar host's details.
#[server(input = GetUrl)]
pub async fn get_stellar_host_detail(
    hostname: String,
) -> Result<Option<StellarHostDetail>, ServerFnError> {
    tracing::info!("get_stellar_host_detail called: hostname={hostname}");
    let state = expect_context::<ApiState>();

    let result = details::get_stellar_host_detail_cached(
        &state.stellarhosts_df,
        &state.host_detail_cache,
        &state.stellarhosts_metadata,
        &hostname,
    )
    .await;
    match result {
        Ok((detail, metadata)) => {
            Ok(Some(StellarHostDetail { metadata, ..detail }))
        }
        Err(details::DetailLookupError::NotFound(_)) => Ok(None),
        Err(error) => {
            tracing::error!("get_stellar_host_detail error: {error}");
            Err(ServerFnError::ServerError(error.to_string()))
        }
    }
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
) -> Result<Option<ExoplanetDetail>, ServerFnError> {
    tracing::info!("get_exoplanet_detail called: pl_name={pl_name}");
    let state = expect_context::<ApiState>();

    let (records, metadata) = match details::get_exoplanet_by_name(
        &state.exoplanets_df,
        &state.exoplanets_metadata,
        &pl_name,
    ) {
        Ok(detail) => detail,
        Err(details::DetailLookupError::NotFound(_)) => return Ok(None),
        Err(error) => {
            tracing::error!("get_exoplanet_detail error: {error}");
            return Err(ServerFnError::ServerError(error.to_string()));
        }
    };

    let canonical =
        exoplanet_canonical::build_canonical_exoplanet(&records, &metadata);

    Ok(Some(ExoplanetDetail {
        selected_record_index: exo_core::selection::exoplanet_record_index(
            &records,
        ),
        pl_name,
        canonical,
        records,
        metadata,
    }))
}
