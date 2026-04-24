use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::components::A;
use leptos_router::lazy_route;

use exo_types::insights::{self, InsightMeta};

use crate::metadata_helpers::{canonical_url, title_with_site};

#[derive(Clone)]
pub struct InsightsLazy;

#[lazy_route]
impl LazyRoute for InsightsLazy {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <InsightsPage/> }.into_any()
    }
}

#[component]
pub fn InsightsPage() -> impl IntoView {
    view! {
        <Title text=insights_title()/>
        <Meta name="description" content=insights_description()/>
        <Link rel="canonical" href=canonical_url("/insights")/>

        <div class="insights-page">
            <div class="insights-page__container">
                <section class="insights-hero">
                    <div class="insights-hero__eyebrow">
                        <span>"Insights"</span>
                        <span>"Ranked lists"</span>
                    </div>

                    <div class="insights-hero__header">
                        <div class="insights-hero__content">
                            <h1 class="insights-hero__title">
                                "Curated exoplanet and stellar-host insights"
                            </h1>
                            <p class="insights-hero__subtitle">
                                "Explore notable planets, host stars, and planetary systems through focused rankings built from the catalog data."
                            </p>
                        </div>

                        <div class="insights-hero__status">
                            <p class="insights-hero__status-label">"Available"</p>
                            <p class="insights-hero__status-value">{format!("{} insights", insights::INSIGHTS.len())}</p>
                        </div>
                    </div>
                </section>

                <section class="insights-section">
                    <div class="insights-section__header">
                        <div>
                            <p class="insights-section__eyebrow">"Browse"</p>
                            <h2 class="insights-section__title">"Exoplanet and stellar-host rankings"</h2>
                        </div>
                        <p class="insights-section__description">
                            "Open a ranking to compare planets, host stars, and system architectures across the archive."
                        </p>
                    </div>

                    <div class="insights-grid">
                        {insights::INSIGHTS.iter().map(|&meta| view! {
                            <InsightCard meta />
                        }).collect::<Vec<_>>()}
                    </div>
                </section>
            </div>
        </div>
    }
}

#[component]
fn InsightCard(meta: &'static InsightMeta) -> impl IntoView {
    let href = format!("/insights/{}", meta.slug);

    view! {
        <A href=href attr:class="insight-card insight-card--live">
            <div class="insight-card__header">
                <p class="insight-card__category">{meta.category}</p>
                <span class="insight-card__badge insight-card__badge--live">"Open"</span>
            </div>
            <h3 class="insight-card__title">{meta.title}</h3>
            <p class="insight-card__body">{meta.description}</p>
            <div class="insight-card__footer">
                <span class="insight-card__kind">{meta.kind}</span>
                <span class="insight-card__action">"View ranking →"</span>
            </div>
        </A>
    }
}

fn insights_title() -> String {
    title_with_site("Insights")
}

fn insights_description() -> String {
    "Browse standalone insight pages for exoplanet and stellar-host rankings, comparisons, and system-level highlights.".to_string()
}
