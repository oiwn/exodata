use super::rows::{dataframe_to_json, distinct_value_count};
use super::tables::{TableQuery, TableResult};
use crate::server::cache::{InsightCache, InsightKind, TableCacheValue};
use polars::prelude::*;

const SOLAR_RADIUS_IN_EARTH_RADII: f64 = 109.076;
const SMALLEST_EXOPLANETS_COLUMNS: &[&str] =
    &["pl_name", "hostname", "pl_rade", "pl_bmasse", "disc_year"];
const LARGEST_EXOPLANETS_COLUMNS: &[&str] =
    &["pl_name", "hostname", "pl_rade", "pl_bmasse", "disc_year"];
const MOST_DISTANT_EXOPLANETS_COLUMNS: &[&str] = &[
    "pl_name",
    "hostname",
    "pl_orbsmax",
    "pl_orbper",
    "disc_year",
];
const NEAREST_STELLAR_HOSTS_COLUMNS: &[&str] =
    &["hostname", "sy_dist", "st_teff", "st_mass", "sy_pnum"];
const LARGEST_PLANET_TO_HOST_RATIO_COLUMNS: &[&str] = &[
    "pl_name",
    "hostname",
    "pl_host_radius_ratio",
    "pl_rade",
    "st_rad",
    "disc_year",
];
const MOST_EQUAL_STAR_PLANET_PAIR_COLUMNS: &[&str] = &[
    "pl_name",
    "hostname",
    "pl_host_radius_ratio",
    "pl_host_radius_ratio_delta",
    "pl_rade",
    "st_rad",
    "disc_year",
];
const HOTTEST_STELLAR_HOSTS_COLUMNS: &[&str] =
    &["hostname", "st_teff", "st_mass", "sy_dist", "sy_pnum"];
const SYSTEMS_WITH_MOST_PLANETS_COLUMNS: &[&str] =
    &["sy_name", "hostname", "sy_pnum", "sy_snum", "sy_dist"];
const BINARY_STAR_SYSTEMS_COLUMNS: &[&str] =
    &["sy_name", "hostname", "sy_pnum", "sy_snum", "sy_dist"];

pub async fn get_smallest_exoplanets_by_radius_cached(
    df: &DataFrame,
    insight_cache: &InsightCache,
) -> TableResult {
    get_cached_insight(
        df,
        insight_cache,
        InsightKind::SmallestExoplanetsByRadius,
        get_smallest_exoplanets_by_radius,
    )
    .await
}

pub async fn get_largest_exoplanets_by_radius_cached(
    df: &DataFrame,
    insight_cache: &InsightCache,
) -> TableResult {
    get_cached_insight(
        df,
        insight_cache,
        InsightKind::LargestExoplanetsByRadius,
        get_largest_exoplanets_by_radius,
    )
    .await
}

pub async fn get_most_distant_exoplanets_cached(
    df: &DataFrame,
    insight_cache: &InsightCache,
) -> TableResult {
    get_cached_insight(
        df,
        insight_cache,
        InsightKind::MostDistantExoplanets,
        get_most_distant_exoplanets,
    )
    .await
}

pub async fn get_nearest_stellar_hosts_cached(
    df: &DataFrame,
    insight_cache: &InsightCache,
) -> TableResult {
    get_cached_insight(
        df,
        insight_cache,
        InsightKind::NearestStellarHosts,
        get_nearest_stellar_hosts,
    )
    .await
}

pub async fn get_largest_planet_to_host_ratios_cached(
    df: &DataFrame,
    insight_cache: &InsightCache,
) -> TableResult {
    get_cached_insight(
        df,
        insight_cache,
        InsightKind::LargestPlanetToHostRatios,
        get_largest_planet_to_host_ratios,
    )
    .await
}

pub async fn get_most_equal_star_planet_pairs_cached(
    df: &DataFrame,
    insight_cache: &InsightCache,
) -> TableResult {
    get_cached_insight(
        df,
        insight_cache,
        InsightKind::MostEqualStarPlanetPairs,
        get_most_equal_star_planet_pairs,
    )
    .await
}

pub async fn get_hottest_stellar_hosts_cached(
    df: &DataFrame,
    insight_cache: &InsightCache,
) -> TableResult {
    get_cached_insight(
        df,
        insight_cache,
        InsightKind::HottestStellarHosts,
        get_hottest_stellar_hosts,
    )
    .await
}

pub async fn get_systems_with_most_planets_cached(
    df: &DataFrame,
    insight_cache: &InsightCache,
) -> TableResult {
    get_cached_insight(
        df,
        insight_cache,
        InsightKind::SystemsWithMostPlanets,
        get_systems_with_most_planets,
    )
    .await
}

pub async fn get_binary_star_systems_cached(
    df: &DataFrame,
    insight_cache: &InsightCache,
) -> TableResult {
    get_cached_insight(
        df,
        insight_cache,
        InsightKind::BinaryStarSystems,
        get_binary_star_systems,
    )
    .await
}

async fn get_cached_insight(
    df: &DataFrame,
    insight_cache: &InsightCache,
    kind: InsightKind,
    query: fn(&DataFrame) -> TableResult,
) -> TableResult {
    if let Some(cached) = insight_cache.get(&kind).await {
        return Ok(table_result_from_cache_value(cached));
    }

    let df = df.clone();
    let (rows, total, total_all, columns) =
        tokio::task::spawn_blocking(move || query(&df))
            .await
            .map_err(|e| format!("Failed to join insight query task: {e}"))??;

    insight_cache
        .insert(
            kind,
            TableCacheValue {
                rows: rows.clone(),
                columns: columns.clone(),
                total,
                total_all,
            },
        )
        .await;

    Ok((rows, total, total_all, columns))
}

fn table_result_from_cache_value(
    cached: TableCacheValue,
) -> (Vec<leptos::serde_json::Value>, usize, usize, Vec<String>) {
    (cached.rows, cached.total, cached.total_all, cached.columns)
}

fn get_smallest_exoplanets_by_radius(df: &DataFrame) -> TableResult {
    get_default_exoplanets_data(
        df,
        fixed_query(10, "pl_rade", "asc", SMALLEST_EXOPLANETS_COLUMNS),
    )
}

fn get_largest_exoplanets_by_radius(df: &DataFrame) -> TableResult {
    get_default_exoplanets_data(
        df,
        fixed_query(10, "pl_rade", "desc", LARGEST_EXOPLANETS_COLUMNS),
    )
}

fn get_most_distant_exoplanets(df: &DataFrame) -> TableResult {
    get_default_exoplanets_data(
        df,
        fixed_query(10, "pl_orbsmax", "desc", MOST_DISTANT_EXOPLANETS_COLUMNS),
    )
}

fn get_nearest_stellar_hosts(df: &DataFrame) -> TableResult {
    get_distinct_stellarhosts_data(
        df,
        fixed_query(10, "sy_dist", "asc", NEAREST_STELLAR_HOSTS_COLUMNS),
    )
}

fn get_largest_planet_to_host_ratios(df: &DataFrame) -> TableResult {
    get_computed_exoplanet_ratio_data(
        df,
        fixed_query(
            10,
            "pl_host_radius_ratio",
            "desc",
            LARGEST_PLANET_TO_HOST_RATIO_COLUMNS,
        ),
    )
}

fn get_most_equal_star_planet_pairs(df: &DataFrame) -> TableResult {
    get_computed_exoplanet_ratio_data(
        df,
        fixed_query(
            10,
            "pl_host_radius_ratio_delta",
            "asc",
            MOST_EQUAL_STAR_PLANET_PAIR_COLUMNS,
        ),
    )
}

fn get_hottest_stellar_hosts(df: &DataFrame) -> TableResult {
    get_distinct_stellarhosts_data(
        df,
        fixed_query(10, "st_teff", "desc", HOTTEST_STELLAR_HOSTS_COLUMNS),
    )
}

fn get_systems_with_most_planets(df: &DataFrame) -> TableResult {
    get_distinct_systems_data(
        df,
        fixed_query(10, "sy_pnum", "desc", SYSTEMS_WITH_MOST_PLANETS_COLUMNS),
    )
}

fn get_binary_star_systems(df: &DataFrame) -> TableResult {
    let working = df
        .clone()
        .lazy()
        .filter(col("sy_snum").eq(lit(2i64)))
        .collect()
        .map_err(|e| format!("Failed to filter binary star systems: {e}"))?;

    get_distinct_systems_data(
        &working,
        fixed_query(10, "sy_pnum", "desc", BINARY_STAR_SYSTEMS_COLUMNS),
    )
}

fn get_distinct_systems_data(df: &DataFrame, query: TableQuery) -> TableResult {
    let TableQuery {
        page,
        limit,
        sort_by,
        order,
        selected_columns,
        filter,
    } = query;

    let sort_col = sort_by.unwrap_or_else(|| "sy_name".to_string());
    let mut working = df.clone();

    let mut columns_to_select = selected_columns.unwrap_or_else(|| {
        vec![
            "sy_name".to_string(),
            "hostname".to_string(),
            "sy_pnum".to_string(),
            "sy_snum".to_string(),
            "sy_dist".to_string(),
        ]
    });

    if !columns_to_select.iter().any(|col| col == "sy_name") {
        columns_to_select.insert(0, "sy_name".to_string());
    }
    if !columns_to_select.iter().any(|col| col == &sort_col) {
        columns_to_select.push(sort_col.clone());
    }

    let valid_columns: Vec<String> = columns_to_select
        .into_iter()
        .filter(|col| working.column(col).is_ok())
        .collect();

    if valid_columns.is_empty() {
        return Err("No valid system columns selected".to_string());
    }
    if working.column("sy_name").is_err() {
        return Err("Missing required column 'sy_name'".to_string());
    }
    if working.column(&sort_col).is_err() {
        return Err(format!("Missing system sort column '{sort_col}'"));
    }

    working = working
        .lazy()
        .select(valid_columns.iter().map(col).collect::<Vec<_>>())
        .filter(col("sy_name").is_not_null())
        .filter(col("sy_name").neq(lit("")))
        .collect()
        .map_err(|e| format!("Failed to select distinct system columns: {e}"))?;

    if let Some(filter_value) = filter {
        let needle = filter_value.trim().to_lowercase();
        if !needle.is_empty() {
            let series = working.column("sy_name").map_err(|e| {
                format!("Failed to read system filter column: {e}")
            })?;
            let utf8 = series
                .str()
                .map_err(|e| format!("Failed to read system name column: {e}"))?;
            let mask: BooleanChunked = utf8
                .into_iter()
                .map(|opt| opt.map(|s| s.to_lowercase().contains(&needle)))
                .collect();
            working = working
                .filter(&mask)
                .map_err(|e| format!("Failed to apply system filter: {e}"))?;
        }
    }

    let total_all = distinct_value_count(&working, "sy_name")?;

    working = working
        .lazy()
        .filter(col(&sort_col).is_not_null())
        .collect()
        .map_err(|e| format!("Failed to filter null system sort values: {e}"))?;

    let descending = order.as_deref().unwrap_or("asc") == "desc";
    let sort_options =
        SortMultipleOptions::new().with_order_descending(descending);

    working = working
        .sort([sort_col.as_str()], sort_options.clone())
        .map_err(|e| format!("Failed to sort distinct systems: {e}"))?;

    working = working
        .unique::<String, String>(
            Some(&["sy_name".to_string()]),
            UniqueKeepStrategy::First,
            None,
        )
        .map_err(|e| format!("Failed to deduplicate systems: {e}"))?;

    working = working
        .sort([sort_col.as_str(), "sy_name"], sort_options)
        .map_err(|e| format!("Failed to resort distinct systems: {e}"))?;

    let total = working.height();
    let page = if page == 0 { 1 } else { page };
    let offset = (page - 1) * limit;

    if offset < working.height() {
        let end = std::cmp::min(offset + limit, working.height());
        working = working.slice(offset as i64, end - offset);
    } else {
        working = working.slice(0, 0);
    }

    let rows = dataframe_to_json(&working)?;
    Ok((rows, total, total_all, valid_columns))
}

fn get_default_exoplanets_data(df: &DataFrame, query: TableQuery) -> TableResult {
    if df.column("default_flag").is_err() {
        return Err(
            "Missing required column 'default_flag' for default exoplanet rows"
                .to_string(),
        );
    }

    let working = df
        .clone()
        .lazy()
        .filter(col("default_flag").eq(lit(1i32)))
        .collect()
        .map_err(|e| format!("Failed to filter default exoplanet rows: {e}"))?;

    get_distinct_exoplanets_data(&working, query)
}

fn get_computed_exoplanet_ratio_data(
    df: &DataFrame,
    query: TableQuery,
) -> TableResult {
    if df.column("default_flag").is_err() {
        return Err(
            "Missing required column 'default_flag' for default exoplanet rows"
                .to_string(),
        );
    }
    for column in ["pl_name", "hostname", "pl_rade", "st_rad"] {
        if df.column(column).is_err() {
            return Err(format!(
                "Missing required column '{column}' for planet-to-host ratio"
            ));
        }
    }

    let TableQuery {
        page,
        limit,
        sort_by,
        order,
        selected_columns,
        filter,
    } = query;
    let sort_col = sort_by.unwrap_or_else(|| "pl_host_radius_ratio".to_string());

    let selected_columns = selected_columns.unwrap_or_else(|| {
        LARGEST_PLANET_TO_HOST_RATIO_COLUMNS
            .iter()
            .map(|column| (*column).to_string())
            .collect()
    });

    let mut working = df
        .clone()
        .lazy()
        .filter(col("default_flag").eq(lit(1i32)))
        .filter(col("pl_name").is_not_null())
        .filter(col("pl_name").neq(lit("")))
        .filter(col("pl_rade").is_not_null())
        .filter(col("pl_rade").gt(lit(0.0)))
        .filter(col("st_rad").is_not_null())
        .filter(col("st_rad").gt(lit(0.0)))
        .with_columns([(col("pl_rade")
            / (col("st_rad") * lit(SOLAR_RADIUS_IN_EARTH_RADII)))
        .alias("pl_host_radius_ratio")])
        .with_columns([(col("pl_host_radius_ratio") - lit(1.0))
            .abs()
            .alias("pl_host_radius_ratio_delta")])
        .select(selected_columns.iter().map(col).collect::<Vec<_>>())
        .collect()
        .map_err(|e| {
            format!("Failed to compute planet-to-host radius ratios: {e}")
        })?;

    if let Some(filter_value) = filter {
        let needle = filter_value.trim().to_lowercase();
        if !needle.is_empty() {
            let series = working
                .column("pl_name")
                .map_err(|e| format!("Failed to read filter column: {e}"))?;
            let utf8 = series
                .str()
                .map_err(|e| format!("Failed to read string column: {e}"))?;
            let mask: BooleanChunked = utf8
                .into_iter()
                .map(|opt| opt.map(|s| s.to_lowercase().contains(&needle)))
                .collect();
            working = working
                .filter(&mask)
                .map_err(|e| format!("Failed to apply filter: {e}"))?;
        }
    }

    let total_all = distinct_value_count(&working, "pl_name")?;

    let descending = order.as_deref().unwrap_or("asc") == "desc";
    let sort_options =
        SortMultipleOptions::new().with_order_descending(descending);

    working = working
        .sort([sort_col.as_str()], sort_options.clone())
        .map_err(|e| format!("Failed to sort planet-to-host ratios: {e}"))?;

    working = working
        .unique::<String, String>(
            Some(&["pl_name".to_string()]),
            UniqueKeepStrategy::First,
            None,
        )
        .map_err(|e| format!("Failed to deduplicate ratio planets: {e}"))?;

    working = working
        .sort([sort_col.as_str(), "pl_name"], sort_options)
        .map_err(|e| format!("Failed to resort planet-to-host ratios: {e}"))?;

    let total = working.height();
    let page = if page == 0 { 1 } else { page };
    let offset = (page - 1) * limit;

    if offset < working.height() {
        let end = std::cmp::min(offset + limit, working.height());
        working = working.slice(offset as i64, end - offset);
    } else {
        working = working.slice(0, 0);
    }

    let rows = dataframe_to_json(&working)?;
    Ok((rows, total, total_all, selected_columns))
}

fn fixed_query(
    limit: usize,
    sort_by: &str,
    order: &str,
    columns: &[&str],
) -> TableQuery {
    TableQuery {
        page: 1,
        limit,
        sort_by: Some(sort_by.to_string()),
        order: Some(order.to_string()),
        selected_columns: Some(
            columns.iter().map(|col| (*col).to_string()).collect(),
        ),
        filter: None,
    }
}

pub fn get_distinct_stellarhosts_data(
    df: &DataFrame,
    query: TableQuery,
) -> TableResult {
    let TableQuery {
        page,
        limit,
        sort_by,
        order,
        selected_columns,
        filter,
    } = query;

    let sort_col = sort_by.unwrap_or_else(|| "hostname".to_string());
    let mut working = df.clone();

    let mut columns_to_select = selected_columns.unwrap_or_else(|| {
        vec![
            "hostname".to_string(),
            "sy_dist".to_string(),
            "st_teff".to_string(),
            "st_mass".to_string(),
            "sy_pnum".to_string(),
        ]
    });

    if !columns_to_select.iter().any(|col| col == "hostname") {
        columns_to_select.insert(0, "hostname".to_string());
    }
    if !columns_to_select.iter().any(|col| col == &sort_col) {
        columns_to_select.push(sort_col.clone());
    }

    let valid_columns: Vec<String> = columns_to_select
        .into_iter()
        .filter(|col| working.column(col).is_ok())
        .collect();

    if valid_columns.is_empty() {
        return Err("No valid columns selected".to_string());
    }
    if working.column("hostname").is_err() {
        return Err("Missing required column 'hostname'".to_string());
    }
    if working.column(&sort_col).is_err() {
        return Err(format!("Missing sort column '{sort_col}'"));
    }

    working = working
        .lazy()
        .select(valid_columns.iter().map(col).collect::<Vec<_>>())
        .filter(col("hostname").is_not_null())
        .filter(col("hostname").neq(lit("")))
        .collect()
        .map_err(|e| {
            format!("Failed to select distinct stellar host columns: {e}")
        })?;

    if let Some(filter_value) = filter {
        let needle = filter_value.trim().to_lowercase();
        if !needle.is_empty() {
            let series = working
                .column("hostname")
                .map_err(|e| format!("Failed to read filter column: {e}"))?;
            let utf8 = series
                .str()
                .map_err(|e| format!("Failed to read string column: {e}"))?;
            let mask: BooleanChunked = utf8
                .into_iter()
                .map(|opt| opt.map(|s| s.to_lowercase().contains(&needle)))
                .collect();
            working = working
                .filter(&mask)
                .map_err(|e| format!("Failed to apply filter: {e}"))?;
        }
    }

    let total_all = distinct_value_count(&working, "hostname")?;

    working = working
        .lazy()
        .filter(col(&sort_col).is_not_null())
        .collect()
        .map_err(|e| format!("Failed to filter null sort values: {e}"))?;

    let descending = order.as_deref().unwrap_or("asc") == "desc";
    let sort_options =
        SortMultipleOptions::new().with_order_descending(descending);

    working = working
        .sort([sort_col.as_str()], sort_options.clone())
        .map_err(|e| format!("Failed to sort distinct stellar hosts: {e}"))?;

    working = working
        .unique::<String, String>(
            Some(&["hostname".to_string()]),
            UniqueKeepStrategy::First,
            None,
        )
        .map_err(|e| format!("Failed to deduplicate stellar hosts: {e}"))?;

    working = working
        .sort([sort_col.as_str(), "hostname"], sort_options)
        .map_err(|e| format!("Failed to resort distinct stellar hosts: {e}"))?;

    let total = working.height();
    let page = if page == 0 { 1 } else { page };
    let offset = (page - 1) * limit;

    if offset < working.height() {
        let end = std::cmp::min(offset + limit, working.height());
        working = working.slice(offset as i64, end - offset);
    } else {
        working = working.slice(0, 0);
    }

    let rows = dataframe_to_json(&working)?;
    Ok((rows, total, total_all, valid_columns))
}

pub fn get_distinct_exoplanets_data(
    df: &DataFrame,
    query: TableQuery,
) -> TableResult {
    let TableQuery {
        page,
        limit,
        sort_by,
        order,
        selected_columns,
        filter,
    } = query;

    let sort_col = sort_by.unwrap_or_else(|| "pl_name".to_string());
    let mut working = df.clone();

    let mut columns_to_select = selected_columns.unwrap_or_else(|| {
        vec![
            "pl_name".to_string(),
            "hostname".to_string(),
            "pl_rade".to_string(),
            "pl_bmasse".to_string(),
            "disc_year".to_string(),
        ]
    });

    if !columns_to_select.iter().any(|col| col == "pl_name") {
        columns_to_select.insert(0, "pl_name".to_string());
    }
    if !columns_to_select.iter().any(|col| col == &sort_col) {
        columns_to_select.push(sort_col.clone());
    }

    let valid_columns: Vec<String> = columns_to_select
        .into_iter()
        .filter(|col| working.column(col).is_ok())
        .collect();

    if valid_columns.is_empty() {
        return Err("No valid columns selected".to_string());
    }
    if working.column("pl_name").is_err() {
        return Err("Missing required column 'pl_name'".to_string());
    }
    if working.column(&sort_col).is_err() {
        return Err(format!("Missing sort column '{sort_col}'"));
    }

    working = working
        .lazy()
        .select(valid_columns.iter().map(col).collect::<Vec<_>>())
        .filter(col("pl_name").is_not_null())
        .filter(col("pl_name").neq(lit("")))
        .collect()
        .map_err(|e| {
            format!("Failed to select distinct exoplanet columns: {e}")
        })?;

    if let Some(filter_value) = filter {
        let needle = filter_value.trim().to_lowercase();
        if !needle.is_empty() {
            let series = working
                .column("pl_name")
                .map_err(|e| format!("Failed to read filter column: {e}"))?;
            let utf8 = series
                .str()
                .map_err(|e| format!("Failed to read string column: {e}"))?;
            let mask: BooleanChunked = utf8
                .into_iter()
                .map(|opt| opt.map(|s| s.to_lowercase().contains(&needle)))
                .collect();
            working = working
                .filter(&mask)
                .map_err(|e| format!("Failed to apply filter: {e}"))?;
        }
    }

    let total_all = distinct_value_count(&working, "pl_name")?;

    working = working
        .lazy()
        .filter(col(&sort_col).is_not_null())
        .collect()
        .map_err(|e| format!("Failed to filter null sort values: {e}"))?;

    let descending = order.as_deref().unwrap_or("asc") == "desc";
    let sort_options =
        SortMultipleOptions::new().with_order_descending(descending);

    working = working
        .sort([sort_col.as_str()], sort_options.clone())
        .map_err(|e| format!("Failed to sort distinct exoplanets: {e}"))?;

    working = working
        .unique::<String, String>(
            Some(&["pl_name".to_string()]),
            UniqueKeepStrategy::First,
            None,
        )
        .map_err(|e| format!("Failed to deduplicate planets: {e}"))?;

    working = working
        .sort([sort_col.as_str(), "pl_name"], sort_options)
        .map_err(|e| format!("Failed to resort distinct exoplanets: {e}"))?;

    let total = working.height();
    let page = if page == 0 { 1 } else { page };
    let offset = (page - 1) * limit;

    if offset < working.height() {
        let end = std::cmp::min(offset + limit, working.height());
        working = working.slice(offset as i64, end - offset);
    } else {
        working = working.slice(0, 0);
    }

    let rows = dataframe_to_json(&working)?;
    Ok((rows, total, total_all, valid_columns))
}
