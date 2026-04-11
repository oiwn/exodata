use leptos::prelude::*;

use crate::table::{TableQueryState, build_table_query};

/// Computes which page numbers to show in the pagination bar.
/// Returns a deduplicated, sorted list that includes:
/// - Page 1 and the last page always
/// - A window of ±2 pages around `current_page`
/// Gaps between non-consecutive entries signal where "…" should be rendered.
pub fn compute_page_window(
    current_page: usize,
    total_pages: usize,
) -> Vec<usize> {
    if total_pages == 0 {
        return vec![];
    }

    let mut pages: Vec<usize> = Vec::new();
    pages.push(1);

    let win_start = current_page.saturating_sub(1).max(2);
    let win_end = (current_page + 1).min(total_pages.saturating_sub(1));
    for p in win_start..=win_end {
        pages.push(p);
    }

    if total_pages > 1 {
        pages.push(total_pages);
    }

    pages.dedup();
    pages
}

#[component]
pub fn PaginationLinks(
    current_page: usize,
    total_pages: usize,
    base_url: String,
    sort_col: Option<String>,
    sort_order: String,
    columns: Vec<String>,
    filter: String,
    /// Called with the target page number on click (JS only). The href still
    /// works for no-JS / crawler navigation regardless.
    #[prop(optional)]
    on_page_change: Option<Callback<usize>>,
) -> impl IntoView {
    let make_url = move |p: usize| {
        let query = TableQueryState::new(
            p,
            sort_col.clone(),
            sort_order.clone(),
            columns.clone(),
            filter.clone(),
        );
        format!("{}?{}", base_url, build_table_query(&query))
    };

    let pages = compute_page_window(current_page, total_pages);

    let mut prev: Option<usize> = None;
    let mut items: Vec<AnyView> = Vec::new();
    for p in pages {
        if let Some(prev_p) = prev {
            if p > prev_p + 1 {
                items.push(
                    view! { <span class="text-slate-600 text-xs px-0.5">"…"</span> }.into_any(),
                );
            }
        }
        let href = make_url(p);
        let is_current = p == current_page;
        let class = if is_current {
            "px-1.5 py-0.5 rounded text-xs font-mono bg-slate-700 text-white"
        } else {
            "px-1.5 py-0.5 rounded text-xs font-mono text-slate-500 hover:text-white hover:bg-slate-800"
        };
        items.push(
            view! {
                <a
                    href=href
                    class=class
                    on:click=move |ev| {
                        if let Some(cb) = on_page_change {
                            ev.prevent_default();
                            cb.run(p);
                        }
                    }
                >{p}</a>
            }
            .into_any(),
        );
        prev = Some(p);
    }

    view! {
        <div class="flex items-center gap-0.5">
            {items}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Unit tests for compute_page_window ---

    #[test]
    fn test_empty_when_total_is_zero() {
        assert!(compute_page_window(1, 0).is_empty());
    }

    #[test]
    fn test_single_page() {
        assert_eq!(compute_page_window(1, 1), vec![1]);
    }

    #[test]
    fn test_small_range_no_ellipsis_needed() {
        // page=3, total=5: ±1 window gives [2,3,4] + anchors → [1,2,3,4,5]
        assert_eq!(compute_page_window(3, 5), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_first_page_of_large_range() {
        // page=1, total=100: ±1 → [1, 2, 100]
        assert_eq!(compute_page_window(1, 100), vec![1, 2, 100]);
    }

    #[test]
    fn test_second_page_of_large_range() {
        // page=2, total=100: ±1 → [1, 2, 3, 100]
        assert_eq!(compute_page_window(2, 100), vec![1, 2, 3, 100]);
    }

    #[test]
    fn test_middle_of_large_range() {
        // page=50, total=100: ±1 → [1, 49, 50, 51, 100]
        assert_eq!(compute_page_window(50, 100), vec![1, 49, 50, 51, 100]);
    }

    #[test]
    fn test_last_page_of_large_range() {
        // page=100, total=100: ±1 → [1, 99, 100]
        assert_eq!(compute_page_window(100, 100), vec![1, 99, 100]);
    }

    #[test]
    fn test_second_to_last_page() {
        // page=99, total=100: ±1 → [1, 98, 99, 100]
        assert_eq!(compute_page_window(99, 100), vec![1, 98, 99, 100]);
    }

    #[test]
    fn test_no_duplicates() {
        let result = compute_page_window(3, 5);
        let mut deduped = result.clone();
        deduped.dedup();
        assert_eq!(result, deduped);
    }

    #[test]
    fn test_result_is_sorted() {
        let result = compute_page_window(50, 200);
        let mut sorted = result.clone();
        sorted.sort();
        assert_eq!(result, sorted);
    }

    // --- SSR rendering tests ---

    #[cfg(feature = "ssr")]
    mod ssr {
        use super::*;

        fn render(current_page: usize, total_pages: usize) -> String {
            view! {
                <PaginationLinks
                    current_page=current_page
                    total_pages=total_pages
                    base_url="/stellarhosts".to_string()
                    sort_col=None
                    sort_order="asc".to_string()
                    columns=vec![]
                    filter="".to_string()
                />
            }
            .into_any()
            .to_html()
        }

        #[test]
        fn test_renders_current_page_link() {
            let html = render(1, 10);
            assert!(html.contains("href=\"/stellarhosts?page=1\""));
        }

        #[test]
        fn test_current_page_has_active_class() {
            let html = render(1, 10);
            assert!(html.contains("bg-slate-700"));
        }

        #[test]
        fn test_renders_last_page_link() {
            let html = render(1, 100);
            assert!(html.contains("href=\"/stellarhosts?page=100\""));
        }

        #[test]
        fn test_renders_ellipsis_for_large_range() {
            let html = render(1, 100);
            assert!(html.contains("…"));
        }

        #[test]
        fn test_no_ellipsis_for_small_range() {
            let html = render(3, 5);
            assert!(!html.contains("…"));
        }

        #[test]
        fn test_sort_params_preserved_in_links() {
            let html = view! {
                <PaginationLinks
                    current_page=1
                    total_pages=10
                    base_url="/stellarhosts".to_string()
                    sort_col=Some("st_mass".to_string())
                    sort_order="desc".to_string()
                    columns=vec![]
                    filter="".to_string()
                />
            }
            .into_any()
            .to_html();

            assert!(html.contains("sort=st_mass"));
            assert!(html.contains("order=desc"));
        }

        #[test]
        fn test_empty_renders_no_links() {
            let html = render(1, 0);
            assert!(!html.contains("<a"));
        }
    }
}
