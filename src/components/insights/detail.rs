use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use leptos_router::lazy_route;

use super::binary_systems::BinaryStarSystemsPage;
use super::crowded_systems::SystemsWithMostPlanetsPage;
use super::distant_exoplanets::MostDistantExoplanetsPage;
use super::equal_star_planet_pairs::MostEqualStarPlanetPairsPage;
use super::hottest_stellar_hosts::HottestStellarHostsPage;
use super::largest_exoplanets::LargestExoplanetsPage;
use super::nearest_stellar_hosts::NearestStellarHostsPage;
use super::planet_host_ratios::LargestPlanetToHostRatiosPage;
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
        canonical_url(&format!("/insights/{}", slug.get_untracked()));

    view! {
        <Link rel="canonical" href=canonical_href/>

        <div class="insights-page">
            <div class="insights-page__container">
                <A href="/insights" attr:class="insights-page__back-link">
                    <span>"←"</span>
                    <span>"Back to Insights"</span>
                </A>

                {move || match slug.get().as_str() {
                    "smallest-exoplanets-radius" => view! { <SmallestExoplanetsPage/> }.into_any(),
                    "largest-exoplanets-radius" => view! { <LargestExoplanetsPage/> }.into_any(),
                    "most-distant-exoplanets" => view! { <MostDistantExoplanetsPage/> }.into_any(),
                    "nearest-stellar-hosts" => view! { <NearestStellarHostsPage/> }.into_any(),
                    "largest-planet-to-host-ratios" => view! { <LargestPlanetToHostRatiosPage/> }.into_any(),
                    "most-equal-star-planet-pairs" => view! { <MostEqualStarPlanetPairsPage/> }.into_any(),
                    "hottest-stellar-hosts" => view! { <HottestStellarHostsPage/> }.into_any(),
                    "systems-with-most-planets" => view! { <SystemsWithMostPlanetsPage/> }.into_any(),
                    "binary-star-systems" => view! { <BinaryStarSystemsPage/> }.into_any(),
                    _ => view! {
                        <Title text=move || title_with_site(&format!("{} Insight", slug.get()))/>
                        <Meta name="description" content="Insights page for ranked exoplanet and stellar-host views."/>
                        <section class="insight-detail-shell">
                            <div class="insight-detail-shell__header">
                                <p class="insights-section__eyebrow">"Not Found"</p>
                                <h1 class="insight-detail-shell__title">"Insight not found"</h1>
                                <p class="insight-detail-shell__intro">
                                    "Use the Insights hub to browse the available exoplanet and stellar-host rankings."
                                </p>
                            </div>
                        </section>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}
