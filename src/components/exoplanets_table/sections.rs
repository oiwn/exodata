use crate::table::{PaginationLinks, TablePaginationState};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn ExoplanetsTableMeta() -> impl IntoView {
    use crate::metadata_helpers::{
        canonical_url, exoplanets_description, exoplanets_title,
    };
    use crate::structured_data::{StructuredData, collection_page_schema};
    use leptos_meta::{Link, Meta, Title};

    view! {
        <Title text=exoplanets_title()/>
        <Meta name="description" content=exoplanets_description()/>
        <Link rel="canonical" href=canonical_url("/exoplanets")/>
        <StructuredData
            value=collection_page_schema(
                "Exoplanets",
                &exoplanets_description(),
                "/exoplanets",
            )
        />
    }
}

#[component]
pub fn ExoplanetsPageShell(children: Children) -> impl IntoView {
    view! {
        <div class="exoplanets-page">
            <div class="exoplanets-page__container">{children()}</div>
        </div>
    }
}

#[component]
pub fn ExoplanetsPageHeader() -> impl IntoView {
    view! {
        <div class="exoplanets-page__header">
            <A href="/" attr:class="exoplanets-page__back-link">
                <span>"←"</span>
                <span>"Back to Overview"</span>
            </A>

            <h1 class="exoplanets-page__title">
                "🪐 Exoplanets Catalog"
            </h1>
            <p class="exoplanets-page__subtitle">
                "Browse the complete database of confirmed exoplanets"
            </p>
        </div>
    }
}

#[component]
pub fn ExoplanetsLoadingFallback() -> impl IntoView {
    view! {
        <div class="exoplanets-loading">
            <div class="exoplanets-loading__spinner">
                <div class="exoplanets-loading__ring"></div>
                <div class="exoplanets-loading__icon">
                    "🪐"
                </div>
            </div>
            <span class="exoplanets-loading__label">"Loading data..."</span>
        </div>
    }
}

#[component]
pub fn ExoplanetsErrorState(error_msg: String) -> impl IntoView {
    view! {
        <div class="exoplanets-error">
            <div class="exoplanets-error__body">
                <span class="exoplanets-error__icon">"⚠️"</span>
                <div>
                    <h3 class="exoplanets-error__title">"Connection Error"</h3>
                    <p class="exoplanets-error__message">{error_msg}</p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ExoplanetsPaginationControls(
    state: TablePaginationState,
    on_prev: Callback<()>,
    on_next: Callback<()>,
    #[prop(optional)] page_links: Option<AnyView>,
) -> impl IntoView {
    view! {
        <div class="exoplanets-pagination">
            <div class="exoplanets-pagination__summary">
                {format!("Showing {} - {} of {} records", state.start, state.end, state.total)}
            </div>

            {page_links}

            <div class="exoplanets-pagination__actions">
                <button
                    class="exoplanets-pagination__button"
                    disabled=!state.can_go_prev
                    on:click=move |_| on_prev.run(())
                >
                    "Previous"
                </button>

                <div class="exoplanets-pagination__status">
                    {format!("Page {} of {}", state.current_page, state.total_pages)}
                </div>

                <button
                    class="exoplanets-pagination__button"
                    disabled=!state.can_go_next
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
            base_url="/exoplanets".to_string()
            sort_col=sort_col
            sort_order=sort_order
            columns=columns
            filter=filter
            on_page_change=on_page_change
        />
    }
    .into_any()
}
