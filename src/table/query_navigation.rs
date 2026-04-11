use leptos_router::NavigateOptions;

use crate::table::build_table_query;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableQueryState {
    pub page: usize,
    pub sort_col: Option<String>,
    pub sort_order: String,
    pub columns: Vec<String>,
    pub filter: String,
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
}

pub fn navigate_table_query(
    navigate: &impl Fn(&str, NavigateOptions),
    base_path: &str,
    query: &TableQueryState,
    options: NavigateOptions,
) {
    let query_string = build_table_query(query);
    let url = format!("{base_path}?{query_string}");
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
}
