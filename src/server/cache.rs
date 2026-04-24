//! Cache types and key normalization helpers for server-side table caching.

use crate::server::functions::StellarHostDetail;
use leptos::serde_json::Value;
use moka::future::Cache;

/// Cache namespace for table datasets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TableKind {
    StellarHosts,
    Exoplanets,
}

/// Canonicalized sort order used in cache keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Stable cache key for table queries.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TableCacheKey {
    pub table: TableKind,
    pub page: usize,
    pub limit: usize,
    pub sort_by: Option<String>,
    pub order: SortOrder,
    pub columns: Option<Vec<String>>,
    pub filter: Option<String>,
}

/// Cached table payload.
#[derive(Clone, Debug)]
pub struct TableCacheValue {
    pub rows: Vec<Value>,
    pub columns: Vec<String>,
    pub total: usize,
    pub total_all: usize,
}

pub type TableCache = Cache<TableCacheKey, TableCacheValue>;
pub type HostDetailCache = Cache<String, StellarHostDetail>;
pub type InsightCache = Cache<String, TableCacheValue>;

/// Build a table cache with max-entry bounds.
pub fn build_table_cache(max_entries: u64) -> TableCache {
    Cache::builder().max_capacity(max_entries).build()
}

/// Build a host-detail cache with max-entry bounds.
pub fn build_host_detail_cache(max_entries: u64) -> HostDetailCache {
    Cache::builder().max_capacity(max_entries).build()
}

/// Build an insight cache with max-entry bounds.
pub fn build_insight_cache(max_entries: u64) -> InsightCache {
    Cache::builder().max_capacity(max_entries).build()
}

/// Normalize raw table query parameters into a stable cache key.
pub fn normalize_table_cache_key(
    table: TableKind,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    columns: Option<Vec<String>>,
    filter: Option<String>,
) -> TableCacheKey {
    let page = if page == 0 { 1 } else { page };
    let sort_by = normalize_optional_token(sort_by);
    let order = normalize_sort_order(order);
    let columns = normalize_columns(columns);
    let filter = normalize_filter(filter);

    TableCacheKey {
        table,
        page,
        limit,
        sort_by,
        order,
        columns,
        filter,
    }
}

fn normalize_sort_order(order: Option<String>) -> SortOrder {
    match normalize_optional_token(order).as_deref() {
        Some("desc") => SortOrder::Desc,
        _ => SortOrder::Asc,
    }
}

fn normalize_optional_token(value: Option<String>) -> Option<String> {
    value
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
}

fn normalize_columns(columns: Option<Vec<String>>) -> Option<Vec<String>> {
    columns
        .map(|cols| {
            cols.into_iter()
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|cols| !cols.is_empty())
}

fn normalize_filter(filter: Option<String>) -> Option<String> {
    normalize_optional_token(filter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_key_canonicalizes_common_variants() {
        let key = normalize_table_cache_key(
            TableKind::Exoplanets,
            0,
            50,
            Some(" Disc_Year ".to_string()),
            Some("DESC".to_string()),
            Some(vec![
                " pl_name ".to_string(),
                "".to_string(),
                "hostname".to_string(),
            ]),
            Some("  KePlEr  ".to_string()),
        );

        assert_eq!(key.page, 1);
        assert_eq!(key.limit, 50);
        assert_eq!(key.sort_by.as_deref(), Some("disc_year"));
        assert_eq!(key.order, SortOrder::Desc);
        assert_eq!(
            key.columns,
            Some(vec!["pl_name".to_string(), "hostname".to_string()])
        );
        assert_eq!(key.filter.as_deref(), Some("kepler"));
    }

    #[test]
    fn normalize_key_defaults_to_asc_and_empty_tokens_none() {
        let key = normalize_table_cache_key(
            TableKind::StellarHosts,
            2,
            50,
            Some("  ".to_string()),
            None,
            Some(vec!["".to_string(), "   ".to_string()]),
            Some(" ".to_string()),
        );

        assert_eq!(key.page, 2);
        assert_eq!(key.sort_by, None);
        assert_eq!(key.order, SortOrder::Asc);
        assert_eq!(key.columns, None);
        assert_eq!(key.filter, None);
    }
}
