use super::sections::{
    ExoplanetsErrorState, ExoplanetsLoadingFallback, ExoplanetsPageHeader,
    ExoplanetsPageShell, ExoplanetsPaginationControls, ExoplanetsTableMeta,
    pagination_links_view,
};
use crate::components::column_selector::ColumnSelector;
use crate::components::loading_overlay::LoadingOverlay;
use crate::metadata::use_app_metadata_store;
use crate::server::functions::get_exoplanets_page;
use crate::table::{
    Table, TablePaginationState, TableQuerySignals, TableQueryState,
    build_column_model, is_err_or_lim, navigate_table_query,
};
use leptos::prelude::*;
use leptos_router::LazyRoute;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};
use leptos_router::lazy_route;

const EXOPLANETS_BASE_PATH: &str = "/exoplanets";
const DEFAULT_EXOPLANETS_COLUMNS: [&str; 7] = [
    "pl_name",
    "hostname",
    "discoverymethod",
    "disc_year",
    "pl_orbper",
    "pl_rade",
    "pl_bmasse",
];

fn navigate_exoplanets(
    navigate: &impl Fn(&str, NavigateOptions),
    query: &TableQueryState,
    options: NavigateOptions,
) {
    navigate_table_query(navigate, EXOPLANETS_BASE_PATH, query, options);
}

#[derive(Clone)]
pub struct ExoplanetsTableLazy;

#[lazy_route]
impl LazyRoute for ExoplanetsTableLazy {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <ExoplanetsTablePage/> }.into_any()
    }
}

#[component]
pub fn ExoplanetsTablePage() -> impl IntoView {
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

    let initial_columns = query_map.with_untracked(|q| {
        q.get("columns")
            .map(|s| {
                s.split(',')
                    .map(|col| col.trim().to_string())
                    .filter(|col| !is_err_or_lim(col))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                DEFAULT_EXOPLANETS_COLUMNS
                    .into_iter()
                    .map(str::to_string)
                    .collect()
            })
    });

    let table_state = TableQuerySignals::new(
        initial_page,
        initial_sort_column,
        initial_sort_order,
        initial_columns,
        initial_filter,
    );
    let (selector_is_open, set_selector_is_open) = signal(false);
    let (is_loading, set_is_loading) = signal(false);
    let (has_loaded, set_has_loaded) = signal(false);

    let app_metadata = use_app_metadata_store();
    let available_columns =
        Signal::derive(move || app_metadata.with(|m| m.exoplanets.clone()));

    let fetch_columns = Signal::derive(move || {
        let all_columns: Vec<String> =
            available_columns.get().keys().cloned().collect();
        build_column_model(&all_columns, &table_state.selected_columns.get())
            .fetch_columns
    });

    let table_resource = Resource::new(
        move || {
            (
                table_state.current_page.get(),
                table_state.sort_column.get(),
                table_state.sort_order.get(),
                fetch_columns.get(),
                table_state.filter_text.get(),
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
            get_exoplanets_page(
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
        let current = table_state.query();
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

    let can_go_prev = move || table_state.current_page.get() > 1;
    let can_go_next = move || table_state.current_page.get() < total_pages();

    let on_sort = Callback::new({
        let navigate = navigate.clone();
        move |column: String| {
            let current_sort_column = table_state.sort_column.get();
            let current_sort_order = table_state.sort_order.get();
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

            table_state.set_sort_column.set(next_sort_column.clone());
            table_state.set_sort_order.set(next_sort_order.clone());
            table_state.set_current_page.set(1);
            let query =
                table_state.query_with_sort(1, next_sort_column, next_sort_order);
            navigate_exoplanets(&navigate, &query, Default::default());
        }
    });

    let on_columns_change = Callback::new({
        let navigate = navigate.clone();
        move |columns: Vec<String>| {
            table_state.set_selected_columns.set(columns.clone());
            table_state.set_current_page.set(1);

            let mut next_sort_column = table_state.sort_column.get();
            if let Some(ref sort_col) = next_sort_column
                && !columns.contains(sort_col)
            {
                next_sort_column = None;
                table_state.set_sort_column.set(None);
            }

            let query =
                table_state.query_with_columns(1, next_sort_column, columns);
            navigate_exoplanets(
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
            table_state.set_filter_text.set(value.clone());
            table_state.set_current_page.set(1);
            let query = table_state.query_with_filter(1, value);
            navigate_exoplanets(
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
            table_state.set_current_page.set(page);
            let query = table_state.query_with_page(page);
            navigate_exoplanets(&navigate, &query, Default::default());
        }
    });

    let on_prev_page = Callback::new({
        move |_| {
            if can_go_prev() {
                on_page_change.run(table_state.current_page.get() - 1);
            }
        }
    });

    let on_next_page = Callback::new({
        move |_| {
            if can_go_next() {
                on_page_change.run(table_state.current_page.get() + 1);
            }
        }
    });

    view! {
        <ExoplanetsTableMeta/>
        <ExoplanetsPageShell>
            <ExoplanetsPageHeader/>

            <ColumnSelector
                available_columns=available_columns
                selected_columns=table_state.selected_columns
                on_change=on_columns_change
                is_open=selector_is_open
                on_toggle=Callback::new(move |state| set_selector_is_open.set(state))
            />

            <div class="exoplanets-page__content">
                <LoadingOverlay loading=show_overlay />
                <Transition fallback=move || view! { <ExoplanetsLoadingFallback/> }>
                    {move || {
                        table_resource.get().map(|result| match result {
                            Ok(data) => {
                                let total = data.total;
                                let page = data.page;
                                let limit = data.limit;
                                let start = (page - 1) * limit + 1;
                                let end = std::cmp::min(page * limit, total);
                                let pagination_state = TablePaginationState::new(
                                    start,
                                    end,
                                    total,
                                    table_state.current_page.get(),
                                    total_pages(),
                                    can_go_prev(),
                                    can_go_next(),
                                );

                                let table_metadata = available_columns.get();
                                let all_columns: Vec<String> =
                                    table_metadata.keys().cloned().collect();
                                let model = build_column_model(
                                    &all_columns,
                                    &table_state.selected_columns.get(),
                                );

                                view! {
                                    <div class="space-y-6">
                                        <ExoplanetsPaginationControls
                                            state=pagination_state
                                            on_prev=on_prev_page
                                            on_next=on_next_page
                                        />

                                        <Table
                                            data=data
                                            on_sort=on_sort
                                            current_sort_column=table_state.sort_column.get()
                                            current_sort_order=table_state.sort_order.get()
                                            column_metadata=table_metadata
                                            display_columns=model.display_columns
                                            column_groups=model.groups
                                            filter_input=table_state.filter_input
                                            set_filter_input=table_state.set_filter_input
                                            on_filter_commit=on_filter_commit
                                            link_column="pl_name".to_string()
                                            link_base="/exoplanets/".to_string()
                                        />

                                        <ExoplanetsPaginationControls
                                            state=pagination_state
                                            on_prev=on_prev_page
                                            on_next=on_next_page
                                            page_links=pagination_links_view(
                                                pagination_state.current_page,
                                                pagination_state.total_pages,
                                                table_state.sort_column.get(),
                                                table_state.sort_order.get(),
                                                table_state.selected_columns.get(),
                                                table_state.filter_text.get(),
                                                on_page_change,
                                            )
                                        />
                                    </div>
                                }
                                .into_any()
                            }
                            Err(err) => view! {
                                <ExoplanetsErrorState
                                    error_msg=format!("Error loading data: {}", err)
                                />
                            }
                            .into_any(),
                        })
                    }}
                </Transition>
            </div>
        </ExoplanetsPageShell>
    }
}
