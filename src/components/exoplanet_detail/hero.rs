use leptos::prelude::*;
use leptos_router::components::A;

use super::format::{format_number, planet_visual_class};
use crate::metadata_helpers::encode_path_segment;
use crate::server::functions::ExoplanetDetail;

#[component]
pub fn PlanetHeroSection(detail: ExoplanetDetail) -> impl IntoView {
    let canonical = &detail.canonical;
    let host = canonical
        .hostname
        .as_ref()
        .and_then(|s| s.value.as_str())
        .map(str::to_owned);
    let discovery_method =
        canonical.discovery_method.as_ref().map(|s| s.value.clone());
    let discovery_year = canonical
        .discovery_year
        .as_ref()
        .map(|s| super::format::format_value(&s.value, ""));
    let radius = canonical.radius.as_ref().map(|s| s.value);
    let mass = canonical.mass.as_ref().map(|s| s.value);
    let orbital_period = canonical.orbital_period.as_ref().map(|s| s.value);
    let equilibrium_temp =
        canonical.equilibrium_temperature.as_ref().map(|s| s.value);
    let visual_class = planet_visual_class(radius, equilibrium_temp);
    let host_href = host
        .as_ref()
        .map(|host| format!("/stellarhosts/{}", encode_path_segment(host)));

    let subtitle_parts = [
        radius.map(|value| format!("{} R⊕", format_number(value))),
        orbital_period.map(|value| format!("{} d orbit", format_number(value))),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let subtitle = if subtitle_parts.is_empty() {
        None
    } else {
        Some(subtitle_parts.join(" • "))
    };
    let fallback_subtitle =
        "Planet profile assembled from archive records".to_string();

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
                        <p class="planet-hero__subtitle">
                            {match (host.clone(), host_href) {
                                (Some(host), Some(href)) => view! {
                                    <span class="planet-hero__host-prefix">"Host star "</span>
                                    <A href=href attr:class="planet-hero__host-link">
                                        {host}
                                    </A>
                                    <span>{subtitle.map(|subtitle| format!(" • {subtitle}")).unwrap_or_default()}</span>
                                }.into_any(),
                                _ => view! {
                                    <span>{subtitle.unwrap_or(fallback_subtitle)}</span>
                                }.into_any(),
                            }}
                        </p>
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
                            hint="default parameter set".to_string()
                        />
                        <HeroStat
                            label="Mass"
                            value=mass.map(|value| format!("{} M⊕", format_number(value))).unwrap_or_else(|| "—".to_string())
                            hint="default parameter set".to_string()
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
