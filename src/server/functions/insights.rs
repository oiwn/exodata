use leptos::prelude::*;
use leptos::server_fn::ServerFnError;
use leptos::server_fn::codec::GetUrl;

#[cfg(feature = "ssr")]
use crate::server::data::insights;
#[cfg(feature = "ssr")]
use crate::server::handlers::ApiState;

use super::TableData;

#[server(input = GetUrl)]
pub async fn get_insight(slug: String) -> Result<TableData, ServerFnError> {
    let state = expect_context::<ApiState>();
    let value = insights::get_insight_cached(
        &state.stellarhosts_df,
        &state.exoplanets_df,
        &state.insight_cache,
        &slug,
    )
    .await
    .map_err(|e: String| -> ServerFnError {
        tracing::error!("insight {slug} error: {e}");
        ServerFnError::ServerError(e)
    })?;

    Ok(TableData {
        rows: value.rows,
        columns: value.columns,
        total: value.total,
        total_all: value.total_all,
        page: 1,
        limit: 10,
    })
}
