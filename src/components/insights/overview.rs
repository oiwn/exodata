use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::components::A;
use leptos_router::lazy_route;

use crate::metadata_helpers::{canonical_url, title_with_site};

const INSIGHTS: &[InsightCardData] = &[
    InsightCardData::live(
        "/insights/smallest-exoplanets-radius",
        "Smallest Exoplanets By Radius",
        "Planetary extremes",
        "Tiny confirmed worlds ordered by radius with host-star context.",
        "Top 10 list",
    ),
    InsightCardData::live(
        "/insights/largest-exoplanets-radius",
        "Largest Exoplanets By Radius",
        "Planetary extremes",
        "Inflated giants and outsized worlds ranked by radius with quick host-star context.",
        "Top 10 list",
    ),
    InsightCardData::live(
        "/insights/hottest-stellar-hosts",
        "Hottest Stellar Hosts",
        "Stellar extremes",
        "Host stars with the highest effective temperatures among confirmed systems.",
        "Top 10 list",
    ),
    InsightCardData::live(
        "/insights/systems-with-most-planets",
        "Planetary Systems With Most Planets",
        "System architecture",
        "System names ordered by the archive-backed confirmed planet count.",
        "Top 10 list",
    ),
    InsightCardData::live(
        "/insights/binary-star-systems",
        "Binary Planetary Systems With Planets",
        "System architecture",
        "Planetary systems where the archive star count identifies two stars.",
        "Top 10 list",
    ),
    InsightCardData::planned(
        "Coldest Exoplanets",
        "Planetary climate",
        "Cold, distant, or weakly irradiated planets ranked by equilibrium temperature.",
        "Top 10 list",
    ),
    InsightCardData::planned(
        "Nearest Stellar Hosts",
        "Distance",
        "Nearby host stars ranked by distance from Earth for quick local-neighborhood browsing.",
        "Top 10 list",
    ),
    InsightCardData::planned(
        "Largest Planet-To-Host Ratios",
        "Relationships",
        "Extreme size-ratio systems highlighting oversized planets around comparatively small stars.",
        "Comparison page",
    ),
    InsightCardData::planned(
        "Most Equal Star-Planet Pairs",
        "Relationships",
        "Systems where planet and host-star sizes sit unusually close together in relative scale.",
        "Comparison page",
    ),
];

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
    let live_count = INSIGHTS.iter().filter(|item| item.href.is_some()).count();

    view! {
        <Title text=insights_title()/>
        <Meta name="description" content=insights_description()/>
        <Link rel="canonical" href=canonical_url("/insights")/>

        <div class="insights-page">
            <div class="insights-page__container">
                <section class="insights-hero">
                    <div class="insights-hero__eyebrow">
                        <span>"Insights"</span>
                        <span>"SEO / GEO"</span>
                    </div>

                    <div class="insights-hero__header">
                        <div class="insights-hero__content">
                            <h1 class="insights-hero__title">
                                "Curated exoplanet and stellar-host insights"
                            </h1>
                            <p class="insights-hero__subtitle">
                                "This hub now routes to a few standalone experiment pages. Each live page owns its own lightweight data fetch and rendering instead of reusing the main table-page UI."
                            </p>
                        </div>

                        <div class="insights-hero__status">
                            <p class="insights-hero__status-label">"Stage 2"</p>
                            <p class="insights-hero__status-value">{format!("{live_count} live insights")}</p>
                        </div>
                    </div>
                </section>

                <section class="insights-section">
                    <div class="insights-section__header">
                        <div>
                            <p class="insights-section__eyebrow">"Insight Pages"</p>
                            <h2 class="insights-section__title">"Live experiments plus planned candidates"</h2>
                        </div>
                        <p class="insights-section__description">
                            "Live cards route to standalone pages. Planned cards stay visible here to shape the wider insight catalog."
                        </p>
                    </div>

                    <div class="insights-grid">
                        {INSIGHTS.iter().map(|item| view! {
                            <InsightCard item=*item />
                        }).collect::<Vec<_>>()}
                    </div>
                </section>
            </div>
        </div>
    }
}

#[component]
fn InsightCard(item: InsightCardData) -> impl IntoView {
    if let Some(href) = item.href {
        view! {
            <A href=href attr:class="insight-card insight-card--live">
                <div class="insight-card__header">
                    <p class="insight-card__category">{item.category}</p>
                    <span class="insight-card__badge insight-card__badge--live">"Live"</span>
                </div>
                <h3 class="insight-card__title">{item.title}</h3>
                <p class="insight-card__body">{item.description}</p>
                <div class="insight-card__footer">
                    <span class="insight-card__kind">{item.kind}</span>
                    <span class="insight-card__action">"Open insight →"</span>
                </div>
            </A>
        }
        .into_any()
    } else {
        view! {
            <article class="insight-card" aria-disabled="true">
                <div class="insight-card__header">
                    <p class="insight-card__category">{item.category}</p>
                    <span class="insight-card__badge">"Coming Soon"</span>
                </div>
                <h3 class="insight-card__title">{item.title}</h3>
                <p class="insight-card__body">{item.description}</p>
                <div class="insight-card__footer">
                    <span class="insight-card__kind">{item.kind}</span>
                </div>
            </article>
        }
        .into_any()
    }
}

#[derive(Clone, Copy)]
struct InsightCardData {
    href: Option<&'static str>,
    title: &'static str,
    category: &'static str,
    description: &'static str,
    kind: &'static str,
}

impl InsightCardData {
    const fn live(
        href: &'static str,
        title: &'static str,
        category: &'static str,
        description: &'static str,
        kind: &'static str,
    ) -> Self {
        Self {
            href: Some(href),
            title,
            category,
            description,
            kind,
        }
    }

    const fn planned(
        title: &'static str,
        category: &'static str,
        description: &'static str,
        kind: &'static str,
    ) -> Self {
        Self {
            href: None,
            title,
            category,
            description,
            kind,
        }
    }
}

fn insights_title() -> String {
    title_with_site("Insights")
}

fn insights_description() -> String {
    "Browse standalone insight pages for exoplanet and stellar-host rankings, comparisons, and system-level highlights.".to_string()
}
