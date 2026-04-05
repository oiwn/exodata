use crate::components::column_selector::ColumnSelector;
use crate::components::loading_overlay::LoadingOverlay;
use crate::metadata::use_app_metadata_store;
use crate::metadata_helpers::{
    canonical_url, stellarhosts_description, stellarhosts_title,
};
use crate::server::functions::get_stellarhosts_page;
use crate::structured_data::{StructuredData, collection_page_schema};
use crate::table::{Table, build_column_model, build_table_query, is_err_or_lim};
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::NavigateOptions;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_query_map};
use leptos_router::lazy_route;

type TableQueryState = (usize, Option<String>, String, Vec<String>, String);

// --- Lazy Route ---

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

// --- Component ---

#[component]
pub fn StellarHostsTablePage() -> impl IntoView {
    // Read URL query parameters
    let query_map = use_query_map();
    let navigate = use_navigate();

    // Initialize state from URL params (or defaults)
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

    // Default columns for stellar hosts
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

    // Reactive state for pagination, sorting, and columns
    let (current_page, set_current_page) = signal(initial_page);
    let (sort_column, set_sort_column) = signal(initial_sort_column);
    let (sort_order, set_sort_order) = signal(initial_sort_order);
    let (selected_columns, set_selected_columns) = signal(initial_columns);
    let (selector_is_open, set_selector_is_open) = signal(false);
    let (filter_text, set_filter_text) = signal(initial_filter.clone());
    let (filter_input, set_filter_input) = signal(initial_filter);

    // Signal to track loading state for overlay (managed manually to avoid Transition disposal panic)
    let (is_loading, set_is_loading) = signal(false);
    // Track if we've loaded data at least once (to avoid showing overlay on initial load)
    let (has_loaded, set_has_loaded) = signal(false);
    let app_metadata = use_app_metadata_store();
    let available_columns =
        Signal::derive(move || app_metadata.with(|m| m.stellarhosts.clone()));

    // Compute columns to fetch (base + err/lim companions)
    let fetch_columns = Signal::derive(move || {
        let all_columns: Vec<String> =
            available_columns.get().keys().cloned().collect();
        build_column_model(&all_columns, &selected_columns.get()).fetch_columns
    });

    // Resource that fetches data when dependencies change
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

    // Track when resource source changes to set loading state
    Effect::new(move |prev: Option<TableQueryState>| {
        let current = (
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

    // Update has_loaded when resource completes successfully.
    Effect::new(move |_| {
        match table_resource.get() {
            Some(Ok(_)) => {
                set_is_loading.set(false);
                if !has_loaded.get() {
                    set_has_loaded.set(true);
                }
            }
            Some(Err(_)) => {
                // Also clear loading on error
                set_is_loading.set(false);
            }
            None => {}
        }
    });

    // Only show overlay when reloading (not on initial load)
    let show_overlay =
        Signal::derive(move || is_loading.get() && has_loaded.get());

    // Derived signals for pagination
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

    // Sorting handler
    let on_sort = Callback::new({
        let navigate = navigate.clone();
        move |column: String| {
            if let Some(current) = sort_column.get() {
                if current == column {
                    // Toggle order or clear
                    match sort_order.get().as_str() {
                        "asc" => set_sort_order.set("desc".to_string()),
                        "desc" => {
                            set_sort_column.set(None);
                            set_sort_order.set("asc".to_string());
                        }
                        _ => {}
                    }
                } else {
                    // New column, start with asc
                    set_sort_column.set(Some(column));
                    set_sort_order.set("asc".to_string());
                }
            } else {
                // No current sort, start with asc
                set_sort_column.set(Some(column));
                set_sort_order.set("asc".to_string());
            }
            // Reset to page 1 when sorting changes
            set_current_page.set(1);
            // Update URL with new state
            let page = current_page.get();
            let sort_col = sort_column.get();
            let order = sort_order.get();
            let cols = selected_columns.get();
            let filter = filter_text.get();
            let query_string = build_table_query(
                page,
                sort_col.as_deref(),
                &order,
                Some(&cols),
                Some(&filter),
            );
            navigate(
                &format!("/stellarhosts?{}", query_string),
                Default::default(),
            );
        }
    });

    // Column selection change handler
    let on_columns_change = Callback::new({
        let navigate = navigate.clone();
        move |columns: Vec<String>| {
            set_selected_columns.set(columns.clone());
            set_current_page.set(1); // Reset to page 1 when columns change

            // Clear sort if the sorted column is no longer in selected columns
            let current_sort = sort_column.get();
            if let Some(ref sort_col) = current_sort
                && !columns.contains(sort_col)
            {
                set_sort_column.set(None);
            }

            // Update URL with new state
            let page = current_page.get();
            let sort_col = sort_column.get();
            let order = sort_order.get();
            let filter = filter_text.get();
            let query_string = build_table_query(
                page,
                sort_col.as_deref(),
                &order,
                Some(&columns),
                Some(&filter),
            );
            navigate(
                &format!("/stellarhosts?{}", query_string),
                NavigateOptions {
                    scroll: false,
                    ..Default::default()
                },
            );
        }
    });

    // Filter commit handler (blur/enter)
    let on_filter_commit = Callback::new({
        let navigate = navigate.clone();
        move |value: String| {
            set_filter_text.set(value.clone());
            set_current_page.set(1);
            let sort_col = sort_column.get();
            let order = sort_order.get();
            let cols = selected_columns.get();
            let query_string = build_table_query(
                1,
                sort_col.as_deref(),
                &order,
                Some(&cols),
                Some(&value),
            );
            navigate(
                &format!("/stellarhosts?{}", query_string),
                NavigateOptions {
                    scroll: false,
                    ..Default::default()
                },
            );
        }
    });

    view! {
        <Title text=stellarhosts_title()/>
        <Meta name="description" content=stellarhosts_description()/>
        <Link rel="canonical" href=canonical_url("/stellarhosts")/>
        <StructuredData
            value=collection_page_schema(
                "Stellar Hosts",
                &stellarhosts_description(),
                "/stellarhosts",
            )
        />
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
            // Header
            <div class="container mx-auto px-4 py-8">
                <div class="mb-8">
                    <A href="/" attr:class="inline-flex items-center gap-2 text-purple-400 hover:text-purple-300 transition-colors mb-4">
                        <span>"←"</span>
                        <span>"Back to Overview"</span>
                    </A>

                    <h1 class="text-4xl md:text-5xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400">
                        "⭐ Stellar Hosts Catalog"
                    </h1>
                    <p class="text-gray-300 mt-2">
                        "Browse the complete database of confirmed stellar host systems"
                    </p>
                </div>

                // Column Selector
                <ColumnSelector
                    available_columns=available_columns
                    selected_columns=selected_columns
                    on_change=on_columns_change
                    is_open=selector_is_open
                    on_toggle=Callback::new(move |state| set_selector_is_open.set(state))
                />

                // Main content with Transition (keeps old content visible while loading)
                <div class="relative">
                    <LoadingOverlay loading=show_overlay />
                    <Transition
                        fallback=move || {
                            view! {
                                <div class="flex flex-col justify-center items-center py-20">
                                    <div class="relative">
                                        <div class="animate-spin rounded-full h-20 w-20 border-t-4 border-b-4 border-purple-500"></div>
                                        <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 text-4xl">
                                            "🪐"
                                        </div>
                                    </div>
                                    <span class="mt-6 text-lg text-gray-300 animate-pulse">"Loading data..."</span>
                                </div>
                            }
                        }
                    >
                        {move || {
                            table_resource.get().map(|result| match result {
                                Ok(data) => {
                                    let total = data.total;
                                    let page = data.page;
                                    let limit = data.limit;
                                    let start = (page - 1) * limit + 1;
                                    let end = std::cmp::min(page * limit, total);

                                    view! {
                                        <div class="space-y-6">
                                            // Pagination controls (top)
                                            <div class="flex flex-col md:flex-row items-center justify-between gap-4 px-4">
                                                <div class="text-sm text-gray-400">
                                                    {format!("Showing {} - {} of {} records", start, end, total)}
                                                </div>

                                                <div class="flex items-center gap-4">
                                                    <button
                                                        class="px-4 py-2 rounded-lg bg-slate-800 text-gray-300 border border-slate-700 hover:bg-slate-700 hover:border-slate-600 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
                                                        disabled=move || !can_go_prev()
                                                        on:click={
                                                            let navigate = navigate.clone();
                                                            move |_| {
                                                                if can_go_prev() {
                                                                    set_current_page.update(|p| *p -= 1);
                                                                    let page = current_page.get();
                                                                    let sort_col = sort_column.get();
                                                                    let order = sort_order.get();
                                                                    let cols = selected_columns.get();
                                                                    let filter = filter_text.get();
                                                                    let query_string = build_table_query(
                                                                        page,
                                                                        sort_col.as_deref(),
                                                                        &order,
                                                                        Some(&cols),
                                                                        Some(&filter),
                                                                    );
                                                                    navigate(&format!("/stellarhosts?{}", query_string), Default::default());
                                                                }
                                                            }
                                                        }
                                                    >
                                                        "Previous"
                                                    </button>

                                                    <div class="text-sm text-gray-300 font-mono">
                                                        {move || format!("Page {} of {}", current_page.get(), total_pages())}
                                                    </div>

                                                    <button
                                                        class="px-4 py-2 rounded-lg bg-slate-800 text-gray-300 border border-slate-700 hover:bg-slate-700 hover:border-slate-600 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
                                                        disabled=move || !can_go_next()
                                                        on:click={
                                                            let navigate = navigate.clone();
                                                            move |_| {
                                                                if can_go_next() {
                                                                    set_current_page.update(|p| *p += 1);
                                                                    let page = current_page.get();
                                                                    let sort_col = sort_column.get();
                                                                    let order = sort_order.get();
                                                                    let cols = selected_columns.get();
                                                                    let filter = filter_text.get();
                                                                    let query_string = build_table_query(
                                                                        page,
                                                                        sort_col.as_deref(),
                                                                        &order,
                                                                        Some(&cols),
                                                                        Some(&filter),
                                                                    );
                                                                    navigate(&format!("/stellarhosts?{}", query_string), Default::default());
                                                                }
                                                            }
                                                        }
                                                    >
                                                        "Next"
                                                    </button>
                                                </div>
                                            </div>

                                            // Table
                                            {
                                                let table_metadata = available_columns.get();
                                                let all_columns: Vec<String> = table_metadata
                                                    .keys()
                                                    .cloned()
                                                    .collect::<Vec<_>>();
                                                let model = build_column_model(
                                                    &all_columns,
                                                    &selected_columns.get(),
                                                );
                                                view! {
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
                                                }
                                            }

                                            // Pagination controls (bottom)
                                            <div class="flex flex-col md:flex-row items-center justify-between gap-4 px-4">
                                                <div class="text-sm text-gray-400">
                                                    {format!("Showing {} - {} of {} records", start, end, total)}
                                                </div>

                                                <div class="flex items-center gap-4">
                                                    <button
                                                        class="px-4 py-2 rounded-lg bg-slate-800 text-gray-300 border border-slate-700 hover:bg-slate-700 hover:border-slate-600 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
                                                        disabled=move || !can_go_prev()
                                                        on:click={
                                                            let navigate = navigate.clone();
                                                            move |_| {
                                                                if can_go_prev() {
                                                                    set_current_page.update(|p| *p -= 1);
                                                                    let page = current_page.get();
                                                                    let sort_col = sort_column.get();
                                                                    let order = sort_order.get();
                                                                    let cols = selected_columns.get();
                                                                    let filter = filter_text.get();
                                                                    let query_string = build_table_query(
                                                                        page,
                                                                        sort_col.as_deref(),
                                                                        &order,
                                                                        Some(&cols),
                                                                        Some(&filter),
                                                                    );
                                                                    navigate(&format!("/stellarhosts?{}", query_string), Default::default());
                                                                }
                                                            }
                                                        }
                                                    >
                                                        "Previous"
                                                    </button>

                                                    <div class="text-sm text-gray-300 font-mono">
                                                        {move || format!("Page {} of {}", current_page.get(), total_pages())}
                                                    </div>

                                                    <button
                                                        class="px-4 py-2 rounded-lg bg-slate-800 text-gray-300 border border-slate-700 hover:bg-slate-700 hover:border-slate-600 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
                                                        disabled=move || !can_go_next()
                                                        on:click={
                                                            let navigate = navigate.clone();
                                                            move |_| {
                                                                if can_go_next() {
                                                                    set_current_page.update(|p| *p += 1);
                                                                    let page = current_page.get();
                                                                    let sort_col = sort_column.get();
                                                                    let order = sort_order.get();
                                                                    let cols = selected_columns.get();
                                                                    let filter = filter_text.get();
                                                                    let query_string = build_table_query(
                                                                        page,
                                                                        sort_col.as_deref(),
                                                                        &order,
                                                                        Some(&cols),
                                                                        Some(&filter),
                                                                    );
                                                                    navigate(&format!("/stellarhosts?{}", query_string), Default::default());
                                                                }
                                                            }
                                                        }
                                                    >
                                                        "Next"
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                                Err(err) => {
                                    let error_msg = format!("Error loading data: {}", err);
                                    view! {
                                        <div class="max-w-2xl mx-auto mt-10 bg-red-900/50 border-2 border-red-500 text-red-100 px-6 py-4 rounded-xl backdrop-blur-sm">
                                            <div class="flex items-center gap-3">
                                                <span class="text-2xl">"⚠️"</span>
                                                <div>
                                                    <h3 class="font-semibold text-lg">"Connection Error"</h3>
                                                    <p class="text-sm text-red-200">{error_msg}</p>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                            })
                        }}
                    </Transition>
                </div>
            </div>
        </div>
    }
}
