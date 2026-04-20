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
        "/insights/most-distant-exoplanets",
        "Most Distant Exoplanets From Their Stars",
        "Orbital architecture",
        "Confirmed planets ranked by semi-major axis instead of inferred temperature.",
        "Top 10 list",
    ),
    InsightCardData::live(
        "/insights/nearest-stellar-hosts",
        "Nearest Stellar Hosts",
        "Distance",
        "Nearby host stars ranked by distance from Earth for quick local-neighborhood browsing.",
        "Top 10 list",
    ),
    InsightCardData::live(
        "/insights/largest-planet-to-host-ratios",
        "Largest Planet-To-Host Ratios",
        "Relationships",
        "Extreme size-ratio systems highlighting oversized planets around comparatively small stars.",
        "Top 10 list",
    ),
    InsightCardData::live(
        "/insights/most-equal-star-planet-pairs",
        "Most Equal Star-Planet Pairs",
        "Relationships",
        "Systems where planet and host-star sizes sit unusually close together in relative scale.",
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
                            <p class="insights-hero__status-value">{format!("{} insights", INSIGHTS.len())}</p>
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
    view! {
        <A href=item.href attr:class="insight-card insight-card--live">
            <div class="insight-card__header">
                <p class="insight-card__category">{item.category}</p>
                <span class="insight-card__badge insight-card__badge--live">"Open"</span>
            </div>
            <h3 class="insight-card__title">{item.title}</h3>
            <p class="insight-card__body">{item.description}</p>
            <div class="insight-card__footer">
                <span class="insight-card__kind">{item.kind}</span>
                <span class="insight-card__action">"View ranking →"</span>
            </div>
        </A>
    }
}

#[derive(Clone, Copy)]
struct InsightCardData {
    href: &'static str,
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
            href,
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
