use super::sections::{
    StellarHostsErrorState, StellarHostsLoadingFallback, StellarHostsPageHeader,
    StellarHostsPageShell, StellarHostsPaginationControls, StellarHostsTableMeta,
    pagination_links_view,
};
use crate::components::column_selector::ColumnSelector;
use crate::components::loading_overlay::LoadingOverlay;
use crate::metadata::use_app_metadata_store;
use crate::server::functions::get_stellarhosts_page;
use crate::table::{
    Table, TableQueryState, build_column_model, is_err_or_lim,
    navigate_table_query,
};
use leptos::prelude::*;
use leptos_router::LazyRoute;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};
use leptos_router::lazy_route;

const STELLARHOSTS_BASE_PATH: &str = "/stellarhosts";

fn navigate_stellarhosts(
    navigate: &impl Fn(&str, NavigateOptions),
    query: &TableQueryState,
    options: NavigateOptions,
) {
    navigate_table_query(navigate, STELLARHOSTS_BASE_PATH, query, options);
}

#[derive(Clone)]
pub struct StellarHostsTableLazy;

#[lazy_route]
impl LazyRoute for StellarHostsTableLazy {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <StellarHostsTablePage/> }.into_any()
    }
}

#[component]
pub fn StellarHostsTablePage() -> impl IntoView {
    let query_map = use_query_map();
    let navigate = use_navigate();

    let initial_page = query_map.with_untracked(|q| {
        q.get("page")
            .and_then(|p| p.parse::<usize>().ok())
            .unwrap_or(1)
    });
    let initial_sort_column =
        query_map.with_untracked(|q| q.get("sort").map(|s| s.to_string()));
    let initial_sort_order = query_map.with_untracked(|q| {
        q.get("order")
            .map(|o| o.to_string())
            .unwrap_or_else(|| "asc".to_string())
    });
    let initial_filter = query_map.with_untracked(|q| {
        q.get("filter").map(|f| f.to_string()).unwrap_or_default()
    });

    let default_columns = vec![
        "hostname".to_string(),
        "sy_dist".to_string(),
        "st_teff".to_string(),
        "st_mass".to_string(),
        "sy_pnum".to_string(),
    ];

    let initial_columns = query_map.with_untracked(|q| {
        q.get("columns")
            .map(|s| {
                s.split(',')
                    .map(|col| col.trim().to_string())
                    .filter(|col| !is_err_or_lim(col))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| default_columns.clone())
    });

    let (current_page, set_current_page) = signal(initial_page);
    let (sort_column, set_sort_column) = signal(initial_sort_column);
    let (sort_order, set_sort_order) = signal(initial_sort_order);
    let (selected_columns, set_selected_columns) = signal(initial_columns);
    let (selector_is_open, set_selector_is_open) = signal(false);
    let (filter_text, set_filter_text) = signal(initial_filter.clone());
    let (filter_input, set_filter_input) = signal(initial_filter);
    let (is_loading, set_is_loading) = signal(false);
    let (has_loaded, set_has_loaded) = signal(false);

    let app_metadata = use_app_metadata_store();
    let available_columns =
        Signal::derive(move || app_metadata.with(|m| m.stellarhosts.clone()));

    let fetch_columns = Signal::derive(move || {
        let all_columns: Vec<String> =
            available_columns.get().keys().cloned().collect();
        build_column_model(&all_columns, &selected_columns.get()).fetch_columns
    });

    let table_resource = Resource::new(
        move || {
            (
                current_page.get(),
                sort_column.get(),
                sort_order.get(),
                fetch_columns.get(),
                filter_text.get(),
            )
        },
        move |(page, sort_col, order, columns, filter)| async move {
            let columns_param = if columns.is_empty() {
                None
            } else {
                Some(columns.join(","))
            };
            let filter_param = if filter.trim().is_empty() {
                None
            } else {
                Some(filter)
            };
            get_stellarhosts_page(
                page,
                50,
                sort_col,
                Some(order),
                columns_param,
                filter_param,
            )
            .await
        },
    );

    Effect::new(move |prev: Option<TableQueryState>| {
        let current = TableQueryState::new(
            current_page.get(),
            sort_column.get(),
            sort_order.get(),
            selected_columns.get(),
            filter_text.get(),
        );
        if let Some(prev_val) = prev
            && prev_val != current
        {
            set_is_loading.set(true);
        }
        current
    });

    Effect::new(move |_| match table_resource.get() {
        Some(Ok(_)) => {
            set_is_loading.set(false);
            if !has_loaded.get() {
                set_has_loaded.set(true);
            }
        }
        Some(Err(_)) => set_is_loading.set(false),
        None => {}
    });

    let show_overlay =
        Signal::derive(move || is_loading.get() && has_loaded.get());

    let total_pages = move || {
        table_resource
            .get()
            .and_then(|res| res.ok())
            .map(|data| {
                if data.limit == 0 {
                    1
                } else {
                    data.total.div_ceil(data.limit)
                }
            })
            .unwrap_or(1)
    };

    let can_go_prev = move || current_page.get() > 1;
    let can_go_next = move || current_page.get() < total_pages();

    let on_sort = Callback::new({
        let navigate = navigate.clone();
        move |column: String| {
            let current_sort_column = sort_column.get();
            let current_sort_order = sort_order.get();
            let (next_sort_column, next_sort_order) =
                if let Some(current) = current_sort_column {
                    if current == column {
                        match current_sort_order.as_str() {
                            "asc" => (Some(current), "desc".to_string()),
                            "desc" => (None, "asc".to_string()),
                            _ => (Some(column), "asc".to_string()),
                        }
                    } else {
                        (Some(column), "asc".to_string())
                    }
                } else {
                    (Some(column), "asc".to_string())
                };

            set_sort_column.set(next_sort_column.clone());
            set_sort_order.set(next_sort_order.clone());
            set_current_page.set(1);
            let query = TableQueryState::new(
                1,
                next_sort_column,
                next_sort_order,
                selected_columns.get(),
                filter_text.get(),
            );
            navigate_stellarhosts(&navigate, &query, Default::default());
        }
    });

    let on_columns_change = Callback::new({
        let navigate = navigate.clone();
        move |columns: Vec<String>| {
            set_selected_columns.set(columns.clone());
            set_current_page.set(1);

            let mut next_sort_column = sort_column.get();
            if let Some(ref sort_col) = next_sort_column
                && !columns.contains(sort_col)
            {
                next_sort_column = None;
                set_sort_column.set(None);
            }

            let query = TableQueryState::new(
                1,
                next_sort_column,
                sort_order.get(),
                columns,
                filter_text.get(),
            );
            navigate_stellarhosts(
                &navigate,
                &query,
                NavigateOptions {
                    scroll: false,
                    ..Default::default()
                },
            );
        }
    });

    let on_filter_commit = Callback::new({
        let navigate = navigate.clone();
        move |value: String| {
            set_filter_text.set(value.clone());
            set_current_page.set(1);
            let query = TableQueryState::new(
                1,
                sort_column.get(),
                sort_order.get(),
                selected_columns.get(),
                value,
            );
            navigate_stellarhosts(
                &navigate,
                &query,
                NavigateOptions {
                    scroll: false,
                    ..Default::default()
                },
            );
        }
    });

    let on_page_change = Callback::new({
        let navigate = navigate.clone();
        move |page: usize| {
            set_current_page.set(page);
            let query = TableQueryState::new(
                page,
                sort_column.get(),
                sort_order.get(),
                selected_columns.get(),
                filter_text.get(),
            );
            navigate_stellarhosts(&navigate, &query, Default::default());
        }
    });

    let on_prev_page = Callback::new({
        move |_| {
            if can_go_prev() {
                on_page_change.run(current_page.get() - 1);
            }
        }
    });

    let on_next_page = Callback::new({
        move |_| {
            if can_go_next() {
                on_page_change.run(current_page.get() + 1);
            }
        }
    });

    view! {
        <StellarHostsTableMeta/>
        <StellarHostsPageShell>
            <StellarHostsPageHeader/>

            <ColumnSelector
                available_columns=available_columns
                selected_columns=selected_columns
                on_change=on_columns_change
                is_open=selector_is_open
                on_toggle=Callback::new(move |state| set_selector_is_open.set(state))
            />

            <div class="stellarhosts-page__content">
                <LoadingOverlay loading=show_overlay />
                <Transition fallback=move || view! { <StellarHostsLoadingFallback/> }>
                    {move || {
                        table_resource.get().map(|result| match result {
                            Ok(data) => {
                                let total = data.total;
                                let page = data.page;
                                let limit = data.limit;
                                let start = (page - 1) * limit + 1;
                                let end = std::cmp::min(page * limit, total);

                                let table_metadata = available_columns.get();
                                let all_columns: Vec<String> =
                                    table_metadata.keys().cloned().collect();
                                let model = build_column_model(
                                    &all_columns,
                                    &selected_columns.get(),
                                );

                                view! {
                                    <div class="space-y-6">
                                        <StellarHostsPaginationControls
                                            start=start
                                            end=end
                                            total=total
                                            current_page=current_page.get()
                                            total_pages=total_pages()
                                            can_go_prev=can_go_prev()
                                            can_go_next=can_go_next()
                                            on_prev=on_prev_page
                                            on_next=on_next_page
                                        />

                                        <Table
                                            data=data
                                            on_sort=on_sort
                                            current_sort_column=sort_column.get()
                                            current_sort_order=sort_order.get()
                                            column_metadata=table_metadata
                                            display_columns=model.display_columns
                                            column_groups=model.groups
                                            filter_input=filter_input
                                            set_filter_input=set_filter_input
                                            on_filter_commit=on_filter_commit
                                            link_column="hostname".to_string()
                                            link_base="/stellarhosts/".to_string()
                                        />

                                        <StellarHostsPaginationControls
                                            start=start
                                            end=end
                                            total=total
                                            current_page=current_page.get()
                                            total_pages=total_pages()
                                            can_go_prev=can_go_prev()
                                            can_go_next=can_go_next()
                                            on_prev=on_prev_page
                                            on_next=on_next_page
                                            page_links=pagination_links_view(
                                                current_page.get(),
                                                total_pages(),
                                                sort_column.get(),
                                                sort_order.get(),
                                                selected_columns.get(),
                                                filter_text.get(),
                                                on_page_change,
                                            )
                                        />
                                    </div>
                                }
                                .into_any()
                            }
                            Err(err) => view! {
                                <StellarHostsErrorState
                                    error_msg=format!("Error loading data: {}", err)
                                />
                            }
                            .into_any(),
                        })
                    }}
                </Transition>
            </div>
        </StellarHostsPageShell>
    }
}
