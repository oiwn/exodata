use leptos::prelude::*;
use leptos::serde_json::Value;

use super::format::format_json_value;
use crate::server::functions::HostPlanets;

#[component]
pub fn PlanetsSection(planets: HostPlanets) -> impl IntoView {
    let planet_count = planets.planets.len();

    view! {
        <section class="host-detail-section">
            <div class="host-detail-section__header">
                <div>
                    <p class="host-detail-section__eyebrow host-detail-section__eyebrow--planets">"System Layout"</p>
                    <h2 class="host-detail-section__title">"Known planets"</h2>
                </div>
                <p class="host-detail-section__description">{format!("{} planets linked to this host", planet_count)}</p>
            </div>

            {if planets.planets.is_empty() {
                view! {
                    <div class="host-planets__empty">
                        "No confirmed planets found for this stellar host."
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="host-planets__grid">
                        {planets.planets.into_iter().map(|planet| view! {
                            <PlanetCard planet=planet />
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn PlanetCard(planet: Value) -> impl IntoView {
    let name = planet
        .get("pl_name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let discovery_method = planet
        .get("discoverymethod")
        .and_then(|v| v.as_str())
        .unwrap_or("—")
        .to_string();

    let metrics = vec![
        (
            "Discovery",
            format_json_value(
                planet.get("disc_year").unwrap_or(&Value::Null),
                "",
            ),
        ),
        (
            "Period",
            format_json_value(
                planet.get("pl_orbper").unwrap_or(&Value::Null),
                "days",
            ),
        ),
        (
            "Radius",
            format_json_value(
                planet.get("pl_rade").unwrap_or(&Value::Null),
                "R⊕",
            ),
        ),
        (
            "Mass",
            format_json_value(
                planet.get("pl_bmasse").unwrap_or(&Value::Null),
                "M⊕",
            ),
        ),
        (
            "Eq. Temp",
            format_json_value(planet.get("pl_eqt").unwrap_or(&Value::Null), "K"),
        ),
    ];

    view! {
        <article class="host-planet-card">
            <div class="host-planet-card__header">
                <div>
                    <h3 class="host-planet-card__title">{name}</h3>
                    <p class="host-planet-card__subtitle">{discovery_method}</p>
                </div>
                <div class="host-planet-card__glyph">
                    "◌"
                </div>
            </div>

            <div class="host-planet-card__metrics">
                {metrics.into_iter().map(|(label, value)| view! {
                    <div>
                        <p class="host-planet-card__metric-label">{label}</p>
                        <p class="host-planet-card__metric-value">{value}</p>
                    </div>
                }).collect::<Vec<_>>()}
            </div>
        </article>
    }
}
