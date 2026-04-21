use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use leptos::server_fn::codec::GetUrl;

#[cfg(feature = "ssr")]
use crate::server::data::tables as table_data;
#[cfg(feature = "ssr")]
use crate::server::handlers::ApiState;

use super::TableData;

/// Server function to fetch paginated stellar hosts data.
#[server(input = GetUrl)]
pub async fn get_stellarhosts_page(
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    columns: Option<String>,
    filter: Option<String>,
) -> Result<TableData, ServerFnError> {
    tracing::info!(
        "get_stellarhosts_page called: page={page} columns={columns:?}"
    );
    let state = expect_context::<ApiState>();
    let selected_columns = parse_columns(columns);

    let value = table_data::get_stellarhosts_data_cached(
        &state.stellarhosts_df,
        &state.table_cache,
        table_data::TableQuery {
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

    tracing::info!("get_stellarhosts_page done: total={}", value.total);
    Ok(TableData {
        rows: value.rows,
        columns: value.columns,
        total: value.total,
        total_all: value.total_all,
        page,
        limit,
    })
}

/// Server function to fetch paginated exoplanets data.
#[server(input = GetUrl)]
pub async fn get_exoplanets_page(
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    columns: Option<String>,
    filter: Option<String>,
) -> Result<TableData, ServerFnError> {
    tracing::info!("get_exoplanets_page called: page={page} columns={columns:?}");
    let state = expect_context::<ApiState>();
    let selected_columns = parse_columns(columns);

    let value = table_data::get_exoplanets_data_cached(
        &state.exoplanets_df,
        &state.table_cache,
        table_data::TableQuery {
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

    tracing::info!("get_exoplanets_page done: total={}", value.total);
    Ok(TableData {
        rows: value.rows,
        columns: value.columns,
        total: value.total,
        total_all: value.total_all,
        page,
        limit,
    })
}

#[cfg(feature = "ssr")]
fn parse_columns(columns: Option<String>) -> Option<Vec<String>> {
    columns.map(|s| {
        s.split(',')
            .map(|col| col.trim().to_string())
            .collect::<Vec<_>>()
    })
}
