use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use leptos_router::lazy_route;

use super::crowded_systems::SystemsWithMostPlanetsPage;
use super::hottest_stellar_hosts::HottestStellarHostsPage;
use super::smallest_exoplanets::SmallestExoplanetsPage;
use crate::metadata_helpers::{
    canonical_url, decode_path_segment, title_with_site,
};

#[derive(Clone)]
pub struct InsightDetailLazy;

#[lazy_route]
impl LazyRoute for InsightDetailLazy {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <InsightDetailPage/> }.into_any()
    }
}

#[component]
pub fn InsightDetailPage() -> impl IntoView {
    let params = use_params_map();
    let slug = Memo::new(move |_| {
        let raw = params.read().get("slug").unwrap_or_default();
        decode_path_segment(&raw)
    });
    let canonical_href =
        canonical_url(&format!("/facts/{}", slug.get_untracked()));

    view! {
        <Link rel="canonical" href=canonical_href/>

        <div class="insights-page">
            <div class="insights-page__container">
                <A href="/facts" attr:class="insights-page__back-link">
                    <span>"←"</span>
                    <span>"Back to Insights"</span>
                </A>

                {move || match slug.get().as_str() {
                    "smallest-exoplanets-radius" => view! { <SmallestExoplanetsPage/> }.into_any(),
                    "hottest-stellar-hosts" => view! { <HottestStellarHostsPage/> }.into_any(),
                    "systems-with-most-planets" => view! { <SystemsWithMostPlanetsPage/> }.into_any(),
                    _ => view! {
                        <Title text=move || title_with_site(&format!("{} Insight", slug.get()))/>
                        <Meta name="description" content="Standalone insights page for ranked exoplanet and stellar-host views."/>
                        <section class="insight-detail-shell">
                            <div class="insight-detail-shell__header">
                                <p class="insights-section__eyebrow">"Not Found"</p>
                                <h1 class="insight-detail-shell__title">"Insight page is not available"</h1>
                                <p class="insight-detail-shell__intro">
                                    "This insight slug is not implemented yet. Use the hub page to browse the currently live experiments."
                                </p>
                            </div>
                        </section>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}
