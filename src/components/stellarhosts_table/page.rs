use crate::components::catalog_table::{
    CatalogTableErrorState, CatalogTableLoadingFallback, CatalogTableMeta,
    CatalogTablePageHeader, CatalogTablePageShell, CatalogTableResult,
    catalog_not_found_view,
};
use crate::components::column_selector::ColumnSelector;
use crate::components::loading_overlay::LoadingOverlay;
use crate::metadata::use_app_metadata_store;
use crate::server::functions::get_stellarhosts_page;
use crate::table::{
    TableQuerySignals, TableQueryState, build_column_model,
    initialize_table_query, navigate_table_query,
};
use leptos::prelude::*;
use leptos_router::LazyRoute;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};
use leptos_router::lazy_route;

const STELLARHOSTS_BASE_PATH: &str = "/stellarhosts";
const DEFAULT_STELLARHOSTS_COLUMNS: [&str; 5] =
    ["hostname", "sy_dist", "st_teff", "st_mass", "sy_pnum"];

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

    let initial = query_map.with_untracked(|q| {
        initialize_table_query(
            q.get("page").as_deref(),
            q.get("sort").as_deref(),
            q.get("order").as_deref(),
            q.get("columns").as_deref(),
            q.get("filter").as_deref(),
            &DEFAULT_STELLARHOSTS_COLUMNS,
        )
    });
    let initial = match initial {
        Ok(initial) => initial,
        Err(_) => return catalog_not_found_view(),
    };
    let canonical_query = initial.canonical_query.clone();

    if let Some(canonical_query) = canonical_query.clone() {
        let navigate = navigate.clone();
        Effect::new(move |_| {
            navigate_stellarhosts(
                &navigate,
                &canonical_query,
                NavigateOptions {
                    replace: true,
                    scroll: false,
                    ..Default::default()
                },
            );
        });
    }

    let table_state = TableQuerySignals::new(
        initial.state.page,
        initial.state.sort_col,
        initial.state.sort_order,
        initial.state.columns,
        initial.state.filter,
    );
    let (selector_is_open, set_selector_is_open) = signal(false);
    let (is_loading, set_is_loading) = signal(false);
    let (has_loaded, set_has_loaded) = signal(false);

    let app_metadata = use_app_metadata_store();
    let available_columns =
        Signal::derive(move || app_metadata.with(|m| m.stellarhosts.clone()));

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
            let query = table_state.query().with_sort_column(column);
            table_state.set_query(query.clone());
            navigate_stellarhosts(&navigate, &query, Default::default());
        }
    });

    let on_columns_change = Callback::new({
        let navigate = navigate.clone();
        move |columns: Vec<String>| {
            let query = table_state.query().with_columns(columns);
            table_state.set_query(query.clone());
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
            let query = table_state.query().with_filter(value);
            table_state.set_query(query.clone());
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
            let query = table_state.query().with_page(page);
            table_state.set_query(query.clone());
            navigate_stellarhosts(&navigate, &query, Default::default());
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
        <CatalogTableMeta
            title=crate::metadata_helpers::stellarhosts_title()
            description=crate::metadata_helpers::stellarhosts_description()
            canonical_path=STELLARHOSTS_BASE_PATH
            collection_name="Stellar Hosts"
        />
        <CatalogTablePageShell>
            <CatalogTablePageHeader
                icon="⭐"
                title="Stellar Hosts Catalog"
                subtitle="Browse the complete database of confirmed stellar host systems"
            />

            <ColumnSelector
                available_columns=available_columns
                selected_columns=table_state.selected_columns
                on_change=on_columns_change
                is_open=selector_is_open
                on_toggle=Callback::new(move |state| set_selector_is_open.set(state))
            />

            <div class="catalog-table-page__content">
                <LoadingOverlay loading=show_overlay />
                <Transition fallback=move || view! { <CatalogTableLoadingFallback/> }>
                    {move || {
                        table_resource.get().map(|result| match result {
                            Ok(data) => view! {
                                <CatalogTableResult
                                    data=data
                                    table_state=table_state
                                    available_columns=available_columns.get()
                                    total_pages=total_pages()
                                    can_go_prev=can_go_prev()
                                    can_go_next=can_go_next()
                                    on_sort=on_sort
                                    on_filter_commit=on_filter_commit
                                    on_prev_page=on_prev_page
                                    on_next_page=on_next_page
                                    on_page_change=on_page_change
                                    base_path=STELLARHOSTS_BASE_PATH
                                    link_column="hostname"
                                    link_base="/stellarhosts/"
                                />
                            }.into_any(),
                            Err(err) => view! {
                                <CatalogTableErrorState
                                    error_msg=format!("Error loading data: {}", err)
                                />
                            }
                            .into_any(),
                        })
                    }}
                </Transition>
            </div>
        </CatalogTablePageShell>
    }
    .into_any()
}
