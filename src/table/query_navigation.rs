use leptos_router::NavigateOptions;

use crate::table::{build_table_query, is_err_or_lim};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableQueryState {
    pub page: usize,
    pub sort_col: Option<String>,
    pub sort_order: String,
    pub columns: Vec<String>,
    pub filter: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableQueryInitialization {
    pub state: TableQueryState,
    pub canonical_query: Option<TableQueryState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableQueryError {
    InvalidPage,
}

impl TableQueryState {
    pub fn new(
        page: usize,
        sort_col: Option<String>,
        sort_order: String,
        columns: Vec<String>,
        filter: String,
    ) -> Self {
        Self {
            page,
            sort_col,
            sort_order,
            columns,
            filter,
        }
    }

    pub fn with_page(&self, page: usize) -> Self {
        Self::new(
            page,
            self.sort_col.clone(),
            self.sort_order.clone(),
            self.columns.clone(),
            self.filter.clone(),
        )
    }

    pub fn with_sort_column(&self, column: String) -> Self {
        let (sort_col, sort_order) = match self.sort_col.as_deref() {
            Some(current) if current == column => {
                match self.sort_order.as_str() {
                    "asc" => (Some(column), "desc".to_string()),
                    "desc" => (None, "asc".to_string()),
                    _ => (Some(column), "asc".to_string()),
                }
            }
            _ => (Some(column), "asc".to_string()),
        };

        Self::new(
            1,
            sort_col,
            sort_order,
            self.columns.clone(),
            self.filter.clone(),
        )
    }

    pub fn with_columns(&self, columns: Vec<String>) -> Self {
        let sort_col = self
            .sort_col
            .as_ref()
            .filter(|sort_col| columns.contains(*sort_col))
            .cloned();

        Self::new(
            1,
            sort_col,
            self.sort_order.clone(),
            columns,
            self.filter.clone(),
        )
    }

    pub fn with_filter(&self, filter: String) -> Self {
        Self::new(
            1,
            self.sort_col.clone(),
            self.sort_order.clone(),
            self.columns.clone(),
            filter,
        )
    }
}

pub fn normalize_table_page(page: usize) -> usize {
    if page == 0 { 1 } else { page }
}

pub fn initialize_table_query(
    page_param: Option<&str>,
    sort_param: Option<&str>,
    order_param: Option<&str>,
    columns_param: Option<&str>,
    filter_param: Option<&str>,
    default_columns: &[&str],
) -> Result<TableQueryInitialization, TableQueryError> {
    let page = match page_param {
        Some(value) => value
            .parse::<usize>()
            .ok()
            .filter(|page| *page > 0)
            .ok_or(TableQueryError::InvalidPage)?,
        None => 1,
    };
    let sort_col = sort_param.map(str::to_string);
    let sort_order = order_param.unwrap_or("asc").to_string();
    let filter = filter_param.unwrap_or_default().to_string();
    let columns = columns_param.map(parse_table_columns).unwrap_or_else(|| {
        default_columns
            .iter()
            .map(|column| (*column).to_string())
            .collect()
    });

    let canonical_query = if page == 1 && page_param.is_some() {
        Some(TableQueryState::new(
            page,
            sort_col.clone(),
            sort_order.clone(),
            columns_param.map(parse_table_columns).unwrap_or_default(),
            filter.clone(),
        ))
    } else {
        None
    };

    Ok(TableQueryInitialization {
        state: TableQueryState::new(page, sort_col, sort_order, columns, filter),
        canonical_query,
    })
}

fn parse_table_columns(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|column| column.trim().to_string())
        .filter(|column| !is_err_or_lim(column))
        .collect()
}

pub fn is_table_page_in_range(page: usize, total: usize, limit: usize) -> bool {
    if page == 0 || limit == 0 {
        return false;
    }

    let total_pages = if total == 0 { 1 } else { total.div_ceil(limit) };
    page <= total_pages
}

pub fn build_table_url(base_path: &str, query: &TableQueryState) -> String {
    let query_string = build_table_query(query);
    if query_string.is_empty() {
        base_path.to_string()
    } else {
        format!("{base_path}?{query_string}")
    }
}

pub fn navigate_table_query(
    navigate: &impl Fn(&str, NavigateOptions),
    base_path: &str,
    query: &TableQueryState,
    options: NavigateOptions,
) {
    let url = build_table_url(base_path, query);
    navigate(&url, options);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn table_query_state_new_sets_all_fields() {
        let query = TableQueryState::new(
            3,
            Some("hostname".to_string()),
            "desc".to_string(),
            vec!["hostname".to_string(), "sy_dist".to_string()],
            "kepler".to_string(),
        );

        assert_eq!(query.page, 3);
        assert_eq!(query.sort_col.as_deref(), Some("hostname"));
        assert_eq!(query.sort_order, "desc");
        assert_eq!(query.columns, vec!["hostname", "sy_dist"]);
        assert_eq!(query.filter, "kepler");
    }

    #[test]
    fn query_transitions_reset_page_and_preserve_unaffected_state() {
        let query = TableQueryState::new(
            3,
            Some("hostname".to_string()),
            "asc".to_string(),
            vec!["hostname".to_string(), "sy_dist".to_string()],
            "alpha".to_string(),
        );

        assert_eq!(query.with_page(2).page, 2);
        assert_eq!(
            query.with_sort_column("hostname".to_string()).sort_order,
            "desc"
        );
        assert_eq!(query.with_sort_column("hostname".to_string()).page, 1);
        assert_eq!(
            query
                .with_sort_column("hostname".to_string())
                .with_sort_column("hostname".to_string())
                .sort_col,
            None
        );
        assert_eq!(
            query
                .with_sort_column("sy_dist".to_string())
                .sort_col
                .as_deref(),
            Some("sy_dist")
        );
        assert_eq!(
            query.with_columns(vec!["sy_dist".to_string()]).sort_col,
            None
        );
        assert_eq!(query.with_filter("beta".to_string()).page, 1);
        assert_eq!(query.with_filter("beta".to_string()).filter, "beta");
    }

    #[test]
    fn navigate_table_query_builds_url_and_passes_options() {
        let captured_url = RefCell::new(None::<String>);
        let captured_scroll = RefCell::new(None::<bool>);
        let query = TableQueryState::new(
            2,
            Some("hostname".to_string()),
            "asc".to_string(),
            vec!["hostname".to_string(), "sy_dist".to_string()],
            "alpha centauri".to_string(),
        );

        navigate_table_query(
            &|url, options| {
                captured_url.replace(Some(url.to_string()));
                captured_scroll.replace(Some(options.scroll));
            },
            "/stellarhosts",
            &query,
            NavigateOptions {
                scroll: false,
                ..Default::default()
            },
        );

        assert_eq!(
            captured_url.into_inner().as_deref(),
            Some(
                "/stellarhosts?page=2&sort=hostname&order=asc&columns=hostname,sy_dist&filter=alpha%20centauri"
            )
        );
        assert_eq!(captured_scroll.into_inner(), Some(false));
    }

    #[test]
    fn build_table_url_omits_empty_query_suffix() {
        let query = TableQueryState::new(
            1,
            None,
            "asc".to_string(),
            vec![],
            "".to_string(),
        );

        assert_eq!(build_table_url("/stellarhosts", &query), "/stellarhosts");
    }

    #[test]
    fn build_table_url_omits_page_one_but_keeps_other_params() {
        let query = TableQueryState::new(
            1,
            Some("disc_year".to_string()),
            "desc".to_string(),
            vec![],
            "kepler".to_string(),
        );

        assert_eq!(
            build_table_url("/exoplanets", &query),
            "/exoplanets?sort=disc_year&order=desc&filter=kepler"
        );
    }

    #[test]
    fn normalize_table_page_maps_zero_to_one() {
        assert_eq!(normalize_table_page(0), 1);
        assert_eq!(normalize_table_page(2), 2);
    }

    #[test]
    fn initialization_applies_defaults_without_a_query() {
        let initial = initialize_table_query(
            None,
            None,
            None,
            None,
            None,
            &["hostname", "sy_dist"],
        )
        .unwrap();

        assert_eq!(initial.state.page, 1);
        assert_eq!(initial.state.sort_order, "asc");
        assert_eq!(initial.state.columns, ["hostname", "sy_dist"]);
        assert!(initial.canonical_query.is_none());
    }

    #[test]
    fn initialization_canonicalizes_page_one_and_filters_companion_columns() {
        let initial = initialize_table_query(
            Some("1"),
            Some("pl_rade"),
            Some("desc"),
            Some("pl_name,pl_rade,pl_radeerr1"),
            Some("kepler"),
            &["pl_name"],
        )
        .unwrap();

        assert_eq!(initial.state.page, 1);
        assert_eq!(initial.state.columns, ["pl_name", "pl_rade"]);
        assert_eq!(
            initial.canonical_query,
            Some(TableQueryState::new(
                1,
                Some("pl_rade".to_string()),
                "desc".to_string(),
                vec!["pl_name".to_string(), "pl_rade".to_string()],
                "kepler".to_string(),
            ))
        );
    }

    #[test]
    fn initialization_rejects_malformed_and_non_positive_pages() {
        let malformed = initialize_table_query(
            Some("not-a-page"),
            None,
            None,
            None,
            None,
            &["pl_name"],
        );
        let zero = initialize_table_query(
            Some("0"),
            None,
            None,
            None,
            None,
            &["pl_name"],
        );

        assert_eq!(malformed, Err(TableQueryError::InvalidPage));
        assert_eq!(zero, Err(TableQueryError::InvalidPage));
    }

    #[test]
    fn page_range_accepts_the_first_empty_page_and_rejects_pages_beyond_the_end()
    {
        assert!(is_table_page_in_range(1, 0, 50));
        assert!(is_table_page_in_range(2, 100, 50));
        assert!(!is_table_page_in_range(3, 100, 50));
        assert!(!is_table_page_in_range(0, 100, 50));
    }
}
