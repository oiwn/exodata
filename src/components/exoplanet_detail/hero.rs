use leptos::prelude::*;

use super::format::{
    first_non_empty_string, format_number, median_numeric_value,
    planet_visual_class,
};
use crate::server::functions::ExoplanetDetail;

#[component]
pub fn PlanetHeroSection(detail: ExoplanetDetail) -> impl IntoView {
    let host = first_non_empty_string(&detail.records, "hostname");
    let discovery_method =
        first_non_empty_string(&detail.records, "discoverymethod");
    let discovery_year = first_non_empty_string(&detail.records, "disc_year");
    let radius = median_numeric_value(&detail.records, "pl_rade");
    let mass = median_numeric_value(&detail.records, "pl_bmasse");
    let orbital_period = median_numeric_value(&detail.records, "pl_orbper");
    let equilibrium_temp = median_numeric_value(&detail.records, "pl_eqt");
    let visual_class = planet_visual_class(radius, equilibrium_temp);

    let subtitle_parts = [
        host.clone(),
        radius.map(|value| format!("{} R⊕", format_number(value))),
        orbital_period.map(|value| format!("{} d orbit", format_number(value))),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let subtitle = if subtitle_parts.is_empty() {
        "Planet profile assembled from archive records".to_string()
    } else {
        subtitle_parts.join(" • ")
    };

    view! {
        <section class="planet-hero">
            <div class="planet-hero__layout">
                <div class="planet-hero__content">
                    <div class="planet-hero__eyebrow">
                        <span>"Exoplanet"</span>
                        <span>"Detail"</span>
                    </div>

                    <div class="planet-hero__heading">
                        <h1 class="planet-hero__title">{detail.pl_name.clone()}</h1>
                        <p class="planet-hero__subtitle">{subtitle}</p>
                    </div>

                    <div class="planet-hero__stats">
                        <HeroStat
                            label="Records"
                            value=detail.records.len().to_string()
                            hint="archive rows".to_string()
                        />
                        <HeroStat
                            label="Discovery"
                            value=discovery_year.unwrap_or_else(|| "—".to_string())
                            hint=discovery_method.unwrap_or_else(|| "method unavailable".to_string())
                        />
                        <HeroStat
                            label="Radius"
                            value=radius.map(|value| format!("{} R⊕", format_number(value))).unwrap_or_else(|| "—".to_string())
                            hint="median non-null value".to_string()
                        />
                        <HeroStat
                            label="Mass"
                            value=mass.map(|value| format!("{} M⊕", format_number(value))).unwrap_or_else(|| "—".to_string())
                            hint="median non-null value".to_string()
                        />
                    </div>
                </div>

                <div class="planet-hero__visual">
                    <PlanetVisual
                        visual_class=visual_class
                        host_label=host.unwrap_or_else(|| "host unknown".to_string())
                        equilibrium_temp=equilibrium_temp
                    />
                </div>
            </div>
        </section>
    }
}

#[component]
fn HeroStat(label: &'static str, value: String, hint: String) -> impl IntoView {
    view! {
        <div class="planet-hero-stat">
            <p class="planet-hero-stat__label">{label}</p>
            <p class="planet-hero-stat__value">{value}</p>
            <p class="planet-hero-stat__hint">{hint}</p>
        </div>
    }
}

#[component]
fn PlanetVisual(
    visual_class: &'static str,
    host_label: String,
    equilibrium_temp: Option<f64>,
) -> impl IntoView {
    let temp_label = equilibrium_temp
        .map(|value| format!("{} K equilibrium", format_number(value)))
        .unwrap_or_else(|| "temperature unconstrained".to_string());

    view! {
        <div class="planet-visual">
            <div class="planet-visual__nebula"></div>
            <div class="planet-visual__orbit"></div>
            <div class=format!("planet-visual__world {visual_class}")></div>
            <div class="planet-visual__spec">
                <p class="planet-visual__spec-label">{host_label}</p>
                <p class="planet-visual__spec-value">{temp_label}</p>
            </div>
        </div>
    }
}
