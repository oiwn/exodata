use super::rows::dataframe_to_json;
use crate::server::cache::{
    TableCache, TableCacheValue, TableKind, normalize_table_cache_key,
};
use polars::prelude::*;

/// Result type for table data operations.
pub type TableResult = Result<TableCacheValue, String>;

/// Table configuration for generic data queries.
pub struct TableConfig<'a> {
    pub default_columns: &'a [&'a str],
}

/// Query parameters for paginated table data requests.
pub struct TableQuery {
    pub page: usize,
    pub limit: usize,
    pub sort_by: Option<String>,
    pub order: Option<String>,
    pub selected_columns: Option<Vec<String>>,
    pub filter: Option<String>,
}

pub fn normalize_table_page(page: usize) -> usize {
    if page == 0 { 1 } else { page }
}

/// Generic table data query with shared pagination/sort/select logic.
pub fn get_table_data(
    df: &DataFrame,
    query: TableQuery,
    config: TableConfig<'_>,
) -> TableResult {
    let TableQuery {
        page,
        limit,
        sort_by,
        order,
        selected_columns,
        filter,
    } = query;
    let mut df = df.clone();

    let columns_to_select: Vec<&str> = if let Some(cols) = &selected_columns {
        cols.iter()
            .filter(|col| df.column(col).is_ok())
            .map(|s| s.as_str())
            .collect()
    } else {
        config.default_columns.to_vec()
    };

    if columns_to_select.is_empty() {
        return Err("No valid columns selected".to_string());
    }

    df = df
        .select(columns_to_select.clone())
        .map_err(|e| format!("Failed to select columns: {}", e))?;

    let total_all = df.height();

    if let Some(filter_value) = filter {
        let needle = filter_value.trim().to_lowercase();
        if !needle.is_empty()
            && let Some(first_col) = columns_to_select.first()
        {
            let series = df
                .column(first_col)
                .map_err(|e| format!("Failed to read filter column: {}", e))?;
            let series = if matches!(series.dtype(), DataType::String) {
                series.clone()
            } else {
                series
                    .cast(&DataType::String)
                    .map_err(|e| format!("Failed to cast filter column: {}", e))?
            };
            let utf8 = series
                .str()
                .map_err(|e| format!("Failed to read string column: {}", e))?;
            let mask: BooleanChunked = utf8
                .iter()
                .map(|opt| opt.map(|s| s.to_lowercase().contains(&needle)))
                .collect();
            df = df
                .filter(&mask)
                .map_err(|e| format!("Failed to apply filter: {}", e))?;
        }
    }

    if let Some(sort_col) = &sort_by
        && columns_to_select.contains(&sort_col.as_str())
    {
        df = df
            .lazy()
            .filter(col(sort_col).is_not_null())
            .collect()
            .map_err(|e| format!("Failed to filter nulls: {}", e))?;

        let descending = order.as_deref().unwrap_or("asc") == "desc";
        let options =
            SortMultipleOptions::new().with_order_descending(descending);

        df = df
            .sort([sort_col.as_str()], options)
            .map_err(|e| format!("Failed to sort: {}", e))?;
    }

    let total = df.height();
    let page = normalize_table_page(page);
    let offset = (page - 1) * limit;

    if offset < df.height() {
        let end = std::cmp::min(offset + limit, df.height());
        df = df.slice(offset as i64, end - offset);
    } else {
        df = df.slice(0, 0);
    }

    let rows = dataframe_to_json(&df)?;
    let columns: Vec<String> =
        columns_to_select.iter().map(|s| (*s).to_string()).collect();

    Ok(TableCacheValue {
        rows,
        columns,
        total,
        total_all,
    })
}

pub fn get_stellarhosts_data(df: &DataFrame, query: TableQuery) -> TableResult {
    let default_columns =
        vec!["hostname", "sy_dist", "st_teff", "st_mass", "sy_pnum"];
    get_table_data(
        df,
        query,
        TableConfig {
            default_columns: &default_columns,
        },
    )
}

pub fn get_exoplanets_data(df: &DataFrame, query: TableQuery) -> TableResult {
    let default_columns = vec![
        "pl_name",
        "hostname",
        "discoverymethod",
        "disc_year",
        "pl_orbper",
        "pl_rade",
        "pl_bmasse",
    ];
    get_table_data(
        df,
        query,
        TableConfig {
            default_columns: &default_columns,
        },
    )
}

fn normalize_exoplanets_website_query(mut query: TableQuery) -> TableQuery {
    if query.sort_by.is_none() {
        query.sort_by = Some("rowupdate".to_string());
        query.order = Some("desc".to_string());
    }
    query
}

pub fn get_exoplanets_website_data(
    df: &DataFrame,
    query: TableQuery,
) -> TableResult {
    let mut query = normalize_exoplanets_website_query(query);
    let mut df = df.clone();
    if query.sort_by.as_deref() == Some("rowupdate") {
        let descending = query.order.as_deref().unwrap_or("asc") == "desc";
        df = df
            .sort(
                ["rowupdate"],
                SortMultipleOptions::new()
                    .with_order_descending(descending)
                    .with_nulls_last(true)
                    .with_maintain_order(true),
            )
            .map_err(|e| format!("Failed to sort update dates: {e}"))?;
        // The date key can be hidden and null dates must survive projection.
        query.sort_by = None;
    }
    if query.selected_columns.is_none() {
        query.selected_columns = Some(
            vec![
                "pl_name",
                "hostname",
                "discoverymethod",
                "disc_year",
                "pl_orbper",
                "pl_rade",
                "pl_bmasse",
                "rowupdate",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        );
    }
    get_exoplanets_data(&df, query)
}

pub async fn get_stellarhosts_data_cached(
    df: &DataFrame,
    table_cache: &TableCache,
    query: TableQuery,
) -> TableResult {
    let key = normalize_table_cache_key(
        TableKind::StellarHosts,
        query.page,
        query.limit,
        query.sort_by.clone(),
        query.order.clone(),
        query.selected_columns.clone(),
        query.filter.clone(),
    );

    if let Some(cached) = table_cache.get(&key).await {
        return Ok(cached);
    }

    let df = df.clone();
    let value =
        tokio::task::spawn_blocking(move || get_stellarhosts_data(&df, query))
            .await
            .map_err(|e| {
                format!("Failed to join stellarhosts blocking task: {}", e)
            })??;

    table_cache.insert(key, value.clone()).await;

    Ok(value)
}

pub async fn get_exoplanets_data_cached(
    df: &DataFrame,
    table_cache: &TableCache,
    query: TableQuery,
) -> TableResult {
    get_exoplanets_data_cached_impl(df, table_cache, query, false).await
}

pub async fn get_exoplanets_website_data_cached(
    df: &DataFrame,
    table_cache: &TableCache,
    query: TableQuery,
) -> TableResult {
    get_exoplanets_data_cached_impl(
        df,
        table_cache,
        normalize_exoplanets_website_query(query),
        true,
    )
    .await
}

async fn get_exoplanets_data_cached_impl(
    df: &DataFrame,
    table_cache: &TableCache,
    query: TableQuery,
    website: bool,
) -> TableResult {
    let key = normalize_table_cache_key(
        if website {
            TableKind::ExoplanetsWebsite
        } else {
            TableKind::Exoplanets
        },
        query.page,
        query.limit,
        query.sort_by.clone(),
        query.order.clone(),
        query.selected_columns.clone(),
        query.filter.clone(),
    );

    if let Some(cached) = table_cache.get(&key).await {
        tracing::debug!("exoplanets cache hit: {key:?}");
        return Ok(cached);
    }

    tracing::debug!("exoplanets cache miss: {key:?}");

    let df = df.clone();
    let value = tokio::task::spawn_blocking(move || {
        if website {
            get_exoplanets_website_data(&df, query)
        } else {
            get_exoplanets_data(&df, query)
        }
    })
    .await
    .map_err(|e| format!("Failed to join exoplanets blocking task: {}", e))??;

    table_cache.insert(key, value.clone()).await;

    Ok(value)
}
