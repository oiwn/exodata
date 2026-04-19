use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use leptos::server_fn::codec::GetUrl;

#[cfg(feature = "ssr")]
use crate::server::data::insights;
#[cfg(feature = "ssr")]
use crate::server::handlers::ApiState;

use super::TableData;

#[server(input = GetUrl)]
pub async fn get_smallest_exoplanets_by_radius_insight()
-> Result<TableData, ServerFnError> {
    let state = expect_context::<ApiState>();
    let (rows, total, total_all, columns) =
        insights::get_smallest_exoplanets_by_radius_cached(
            &state.exoplanets_df,
            &state.insight_cache,
        )
        .await
        .map_err(|e: String| -> ServerFnError {
            tracing::error!("smallest exoplanets insight error: {e}");
            ServerFnError::ServerError(e)
        })?;

    Ok(TableData {
        rows,
        columns,
        total,
        total_all,
        page: 1,
        limit: 10,
    })
}

#[server(input = GetUrl)]
pub async fn get_largest_exoplanets_by_radius_insight()
-> Result<TableData, ServerFnError> {
    let state = expect_context::<ApiState>();
    let (rows, total, total_all, columns) =
        insights::get_largest_exoplanets_by_radius_cached(
            &state.exoplanets_df,
            &state.insight_cache,
        )
        .await
        .map_err(|e: String| -> ServerFnError {
            tracing::error!("largest exoplanets insight error: {e}");
            ServerFnError::ServerError(e)
        })?;

    Ok(TableData {
        rows,
        columns,
        total,
        total_all,
        page: 1,
        limit: 10,
    })
}

#[server(input = GetUrl)]
pub async fn get_hottest_stellar_hosts_insight()
-> Result<TableData, ServerFnError> {
    let state = expect_context::<ApiState>();
    let (rows, total, total_all, columns) =
        insights::get_hottest_stellar_hosts_cached(
            &state.stellarhosts_df,
            &state.insight_cache,
        )
        .await
        .map_err(|e: String| -> ServerFnError {
            tracing::error!("hottest stellar hosts insight error: {e}");
            ServerFnError::ServerError(e)
        })?;

    Ok(TableData {
        rows,
        columns,
        total,
        total_all,
        page: 1,
        limit: 10,
    })
}

#[server(input = GetUrl)]
pub async fn get_systems_with_most_planets_insight()
-> Result<TableData, ServerFnError> {
    let state = expect_context::<ApiState>();
    let (rows, total, total_all, columns) =
        insights::get_systems_with_most_planets_cached(
            &state.stellarhosts_df,
            &state.insight_cache,
        )
        .await
        .map_err(|e: String| -> ServerFnError {
            tracing::error!("systems with most planets insight error: {e}");
            ServerFnError::ServerError(e)
        })?;

    Ok(TableData {
        rows,
        columns,
        total,
        total_all,
        page: 1,
        limit: 10,
    })
}

#[server(input = GetUrl)]
pub async fn get_binary_star_systems_insight() -> Result<TableData, ServerFnError>
{
    let state = expect_context::<ApiState>();
    let (rows, total, total_all, columns) =
        insights::get_binary_star_systems_cached(
            &state.stellarhosts_df,
            &state.insight_cache,
        )
        .await
        .map_err(|e: String| -> ServerFnError {
            tracing::error!("binary star systems insight error: {e}");
            ServerFnError::ServerError(e)
        })?;

    Ok(TableData {
        rows,
        columns,
        total,
        total_all,
        page: 1,
        limit: 10,
    })
}
