use leptos::prelude::*;
use leptos::serde_json::Value;

use super::format::format_json_value;
use crate::server::functions::HostPlanets;

#[component]
pub fn PlanetsSection(planets: HostPlanets) -> impl IntoView {
    let planet_count = planets.planets.len();

    view! {
        <section class="space-y-6">
            <div class="flex items-end justify-between gap-4">
                <div>
                    <p class="text-sm uppercase tracking-[0.18em] text-sky-300">"System Layout"</p>
                    <h2 class="mt-2 text-3xl font-semibold text-white">"Known planets"</h2>
                </div>
                <p class="text-sm text-slate-400">{format!("{} planets linked to this host", planet_count)}</p>
            </div>

            {if planets.planets.is_empty() {
                view! {
                    <div class="rounded-[1.5rem] border border-slate-800 bg-slate-950/70 px-6 py-10 text-center text-slate-400">
                        "No confirmed planets found for this stellar host."
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
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
        <article class="rounded-[1.5rem] border border-slate-800 bg-slate-950/70 p-5 shadow-lg shadow-slate-950/30">
            <div class="flex items-start justify-between gap-4">
                <div>
                    <h3 class="text-xl font-semibold text-white">{name}</h3>
                    <p class="mt-1 text-sm text-slate-400">{discovery_method}</p>
                </div>
                <div class="h-12 w-12 rounded-full bg-sky-400/10 text-center text-2xl leading-[3rem] text-sky-300">
                    "◌"
                </div>
            </div>

            <div class="mt-5 grid gap-3 sm:grid-cols-2">
                {metrics.into_iter().map(|(label, value)| view! {
                    <div>
                        <p class="text-xs uppercase tracking-[0.16em] text-slate-500">{label}</p>
                        <p class="mt-1 text-sm text-white">{value}</p>
                    </div>
                }).collect::<Vec<_>>()}
            </div>
        </article>
    }
}
