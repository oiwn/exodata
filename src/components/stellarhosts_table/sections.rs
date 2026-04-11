use crate::table::PaginationLinks;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn StellarHostsTableMeta() -> impl IntoView {
    use crate::metadata_helpers::{
        canonical_url, stellarhosts_description, stellarhosts_title,
    };
    use crate::structured_data::{StructuredData, collection_page_schema};
    use leptos_meta::{Link, Meta, Title};

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
    }
}

#[component]
pub fn StellarHostsPageShell(children: Children) -> impl IntoView {
    view! {
        <div class="stellarhosts-page">
            <div class="stellarhosts-page__container">{children()}</div>
        </div>
    }
}

#[component]
pub fn StellarHostsPageHeader() -> impl IntoView {
    view! {
        <div class="stellarhosts-page__header">
            <A href="/" attr:class="stellarhosts-page__back-link">
                <span>"←"</span>
                <span>"Back to Overview"</span>
            </A>

            <h1 class="stellarhosts-page__title">
                "⭐ Stellar Hosts Catalog"
            </h1>
            <p class="stellarhosts-page__subtitle">
                "Browse the complete database of confirmed stellar host systems"
            </p>
        </div>
    }
}

#[component]
pub fn StellarHostsLoadingFallback() -> impl IntoView {
    view! {
        <div class="stellarhosts-loading">
            <div class="stellarhosts-loading__spinner">
                <div class="stellarhosts-loading__ring"></div>
                <div class="stellarhosts-loading__icon">
                    "🪐"
                </div>
            </div>
            <span class="stellarhosts-loading__label">"Loading data..."</span>
        </div>
    }
}

#[component]
pub fn StellarHostsErrorState(error_msg: String) -> impl IntoView {
    view! {
        <div class="stellarhosts-error">
            <div class="stellarhosts-error__body">
                <span class="stellarhosts-error__icon">"⚠️"</span>
                <div>
                    <h3 class="stellarhosts-error__title">"Connection Error"</h3>
                    <p class="stellarhosts-error__message">{error_msg}</p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn StellarHostsPaginationControls(
    start: usize,
    end: usize,
    total: usize,
    current_page: usize,
    total_pages: usize,
    can_go_prev: bool,
    can_go_next: bool,
    on_prev: Callback<()>,
    on_next: Callback<()>,
    #[prop(optional)] page_links: Option<AnyView>,
) -> impl IntoView {
    view! {
        <div class="stellarhosts-pagination">
            <div class="stellarhosts-pagination__summary">
                {format!("Showing {} - {} of {} records", start, end, total)}
            </div>

            {page_links}

            <div class="stellarhosts-pagination__actions">
                <button
                    class="stellarhosts-pagination__button"
                    disabled=!can_go_prev
                    on:click=move |_| on_prev.run(())
                >
                    "Previous"
                </button>

                <div class="stellarhosts-pagination__status">
                    {format!("Page {} of {}", current_page, total_pages)}
                </div>

                <button
                    class="stellarhosts-pagination__button"
                    disabled=!can_go_next
                    on:click=move |_| on_next.run(())
                >
                    "Next"
                </button>
            </div>
        </div>
    }
}

pub fn pagination_links_view(
    current_page: usize,
    total_pages: usize,
    sort_col: Option<String>,
    sort_order: String,
    columns: Vec<String>,
    filter: String,
    on_page_change: Callback<usize>,
) -> AnyView {
    view! {
        <PaginationLinks
            current_page=current_page
            total_pages=total_pages
            base_url="/stellarhosts".to_string()
            sort_col=sort_col
            sort_order=sort_order
            columns=columns
            filter=filter
            on_page_change=on_page_change
        />
    }
    .into_any()
}
