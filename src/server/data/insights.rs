use super::rows::{dataframe_to_json, distinct_value_count};
use super::tables::{TableQuery, TableResult};
use polars::prelude::*;

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
