use leptos::prelude::*;

use super::format::{alias_label, format_numeric_primary, format_stable_value};
use super::star_color::star_visual_tokens;
use super::star_visual::HostStarVisual;
use crate::server::functions::StellarHostDetail;

#[component]
pub fn HostHeroSection(host: StellarHostDetail) -> impl IntoView {
    let spectype = host
        .star
        .spectype
        .as_ref()
        .map(|value| value.value.clone())
        .unwrap_or_else(|| "Unclassified star".to_string());
    let distance = host.system.distance.as_ref().map(format_numeric_primary);
    let planet_count = host.system.planet_count.as_ref().map(format_stable_value);
    let star_tokens =
        star_visual_tokens(host.star.teff.as_ref().map(|teff| teff.value));
    let aliases = host
        .identity
        .aliases
        .iter()
        .filter(|(_, values)| !values.is_empty())
        .map(|(label, values)| {
            format!("{}: {}", alias_label(label), values.join(", "))
        })
        .collect::<Vec<_>>();

    view! {
        <section class="host-hero">
            <div class="host-hero__layout">
                <div class="host-hero__content">
                    <div class="host-hero__eyebrow">
                        <span>"Stellar Host"</span>
                        <span>"Profile"</span>
                    </div>
                    <div class="host-hero__heading">
                        <h1 class="host-hero__title">
                            {host.hostname.clone()}
                        </h1>
                        <p class="host-hero__subtitle">
                            {spectype}
                            {distance.map(|value| format!(" • {}", value)).unwrap_or_default()}
                        </p>
                    </div>

                    <div class="host-hero__stats">
                        <HeroStat
                            label="Records"
                            value=host.provenance.record_count.to_string()
                            hint="literature rows"
                        />
                        <HeroStat
                            label="Planets"
                            value=planet_count.unwrap_or_else(|| "—".to_string())
                            hint="system count"
                        />
                        <HeroStat
                            label="References"
                            value=(host.provenance.stellar_refs.len() + host.provenance.system_refs.len()).to_string()
                            hint="distinct source labels"
                        />
                    </div>

                    {if aliases.is_empty() {
                        view! { <div></div> }.into_any()
                    } else {
                        view! {
                            <div class="host-hero__aliases">
                                {aliases.into_iter().map(|alias| view! {
                                    <span class="host-hero__alias">
                                        {alias}
                                    </span>
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }}
                </div>

                <div class="host-hero__visual">
                    <HostStarVisual tokens=star_tokens />
                </div>
            </div>
        </section>
    }
}

#[component]
fn HeroStat(
    label: &'static str,
    value: String,
    hint: &'static str,
) -> impl IntoView {
    view! {
        <div class="host-hero-stat">
            <p class="host-hero-stat__label">{label}</p>
            <p class="host-hero-stat__value">{value}</p>
            <p class="host-hero-stat__hint">{hint}</p>
        </div>
    }
}
