use super::rows::dataframe_to_json;
use super::tables::TableResult;
use crate::server::cache::{InsightCache, TableCacheValue};
use exo_core::insights::{self, InsightInput};
use polars::prelude::*;

pub async fn get_insight_cached(
    stellarhosts_df: &DataFrame,
    exoplanets_df: &DataFrame,
    insight_cache: &InsightCache,
    slug: &str,
) -> TableResult {
    if let Some(cached) = insight_cache.get(slug).await {
        return Ok(cached);
    }

    let slug = slug.to_string();
    let slug_for_query = slug.clone();
    let stellarhosts_df = stellarhosts_df.clone();
    let exoplanets_df = exoplanets_df.clone();
    let insight_data = tokio::task::spawn_blocking(move || {
        insights::run_insight(
            InsightInput {
                stellarhosts: &stellarhosts_df,
                exoplanets: &exoplanets_df,
            },
            &slug_for_query,
        )
    })
    .await
    .map_err(|e| format!("Failed to join insight query task: {e}"))?
    .map_err(|e| e.to_string())?;

    let rows = dataframe_to_json(&insight_data.frame)?;
    let total = insight_data.frame.height();
    let cache_value = TableCacheValue {
        rows: rows.clone(),
        columns: insight_data.columns.clone(),
        total,
        total_all: total,
    };

    insight_cache.insert(slug, cache_value.clone()).await;

    Ok(cache_value)
}
