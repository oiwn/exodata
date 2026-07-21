use crate::structured_data::{StructuredData, collection_page_schema};
use crate::table::{
    PaginationLinks, Table, TablePaginationState, TableQuerySignals,
    build_column_model,
};
use crate::{
    error_template::{AppError, ErrorTemplate},
    server::functions::TableData,
};
use exo_types::metadata::ColumnMetadata;
use leptos::error::Errors;
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::components::A;
use std::collections::HashMap;

#[component]
pub fn CatalogTableMeta(
    title: String,
    description: String,
    canonical_path: &'static str,
    collection_name: &'static str,
) -> impl IntoView {
    let canonical_url = crate::metadata_helpers::canonical_url(canonical_path);
    let structured_description = description.clone();

    view! {
        <Title text=title/>
        <Meta name="description" content=description/>
        <Link rel="canonical" href=canonical_url/>
        <StructuredData
            value=collection_page_schema(
                collection_name,
                &structured_description,
                canonical_path,
            )
        />
    }
}

#[component]
pub fn CatalogTablePageShell(children: Children) -> impl IntoView {
    view! {
        <div class="catalog-table-page">
            <div class="catalog-table-page__container">{children()}</div>
        </div>
    }
}

#[component]
pub fn CatalogTablePageHeader(
    icon: &'static str,
    title: &'static str,
    subtitle: &'static str,
) -> impl IntoView {
    view! {
        <div class="catalog-table-page__header">
            <A href="/" attr:class="catalog-table-page__back-link">
                <span>"←"</span>
                <span>"Back to Overview"</span>
            </A>

            <h1 class="catalog-table-page__title">
                {icon} " " {title}
            </h1>
            <p class="catalog-table-page__subtitle">{subtitle}</p>
        </div>
    }
}

#[component]
pub fn CatalogTableLoadingFallback() -> impl IntoView {
    view! {
        <div class="catalog-table-loading">
            <div class="catalog-table-loading__spinner">
                <div class="catalog-table-loading__ring"></div>
                <div class="catalog-table-loading__icon">"🪐"</div>
            </div>
            <span class="catalog-table-loading__label">"Loading data..."</span>
        </div>
    }
}

#[component]
pub fn CatalogTableErrorState(error_msg: String) -> impl IntoView {
    view! {
        <div class="catalog-table-error">
            <div class="catalog-table-error__body">
                <span class="catalog-table-error__icon">"⚠️"</span>
                <div>
                    <h3 class="catalog-table-error__title">"Connection Error"</h3>
                    <p class="catalog-table-error__message">{error_msg}</p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn CatalogTablePaginationControls(
    state: TablePaginationState,
    on_prev: Callback<()>,
    on_next: Callback<()>,
    #[prop(optional)] page_links: Option<AnyView>,
) -> impl IntoView {
    view! {
        <div class="catalog-table-pagination">
            <div class="catalog-table-pagination__summary">
                {format!("Showing {} - {} of {} records", state.start, state.end, state.total)}
            </div>

            {page_links}

            <div class="catalog-table-pagination__actions">
                <button
                    class="catalog-table-pagination__button"
                    disabled=!state.can_go_prev
                    on:click=move |_| on_prev.run(())
                >
                    "Previous"
                </button>

                <div class="catalog-table-pagination__status">
                    {format!("Page {} of {}", state.current_page, state.total_pages)}
                </div>

                <button
                    class="catalog-table-pagination__button"
                    disabled=!state.can_go_next
                    on:click=move |_| on_next.run(())
                >
                    "Next"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn CatalogTableResult(
    data: TableData,
    table_state: TableQuerySignals,
    available_columns: HashMap<String, ColumnMetadata>,
    total_pages: usize,
    can_go_prev: bool,
    can_go_next: bool,
    on_sort: Callback<String>,
    on_filter_commit: Callback<String>,
    on_prev_page: Callback<()>,
    on_next_page: Callback<()>,
    on_page_change: Callback<usize>,
    base_path: &'static str,
    link_column: &'static str,
    link_base: &'static str,
) -> AnyView {
    if catalog_page_is_out_of_range(&data) {
        return catalog_not_found_view();
    }

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
        total_pages,
        can_go_prev,
        can_go_next,
    );
    let all_columns: Vec<String> = available_columns.keys().cloned().collect();
    let model =
        build_column_model(&all_columns, &table_state.selected_columns.get());

    view! {
        <div class="space-y-6">
            <CatalogTablePaginationControls
                state=pagination_state
                on_prev=on_prev_page
                on_next=on_next_page
            />

            <Table
                data=data
                on_sort=on_sort
                current_sort_column=table_state.sort_column.get()
                current_sort_order=table_state.sort_order.get()
                column_metadata=available_columns
                display_columns=model.display_columns
                column_groups=model.groups
                filter_input=table_state.filter_input
                set_filter_input=table_state.set_filter_input
                on_filter_commit=on_filter_commit
                link_column=link_column.to_string()
                link_base=link_base.to_string()
            />

            <CatalogTablePaginationControls
                state=pagination_state
                on_prev=on_prev_page
                on_next=on_next_page
                page_links=pagination_links_view(
                    base_path,
                    pagination_state,
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

pub fn pagination_links_view(
    base_url: &'static str,
    pagination_state: TablePaginationState,
    sort_col: Option<String>,
    sort_order: String,
    columns: Vec<String>,
    filter: String,
    on_page_change: Callback<usize>,
) -> AnyView {
    view! {
        <PaginationLinks
            current_page=pagination_state.current_page
            total_pages=pagination_state.total_pages
            base_url=base_url.to_string()
            sort_col=sort_col
            sort_order=sort_order
            columns=columns
            filter=filter
            on_page_change=on_page_change
        />
    }
    .into_any()
}

pub fn catalog_not_found_view() -> AnyView {
    let mut errors = Errors::default();
    errors.insert_with_default_key(AppError::NotFound);
    view! { <ErrorTemplate outside_errors=errors/> }.into_any()
}

pub fn catalog_page_is_out_of_range(data: &TableData) -> bool {
    !crate::table::is_table_page_in_range(data.page, data.total, data.limit)
}
