use leptos::prelude::*;
use leptos::serde_json::Value;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::server::functions::{
    get_planets_for_host, get_stellar_host_detail, HostPlanets, StellarHostDetail,
};

#[component]
pub fn StellarHostDetailPage() -> impl IntoView {
    let params = use_params_map();
    let hostname = move || {
        params
            .read()
            .get("hostname")
            .unwrap_or_default()
            .replace("%20", " ")
    };

    // Fetch stellar host details
    let host_resource = Resource::new(
        move || hostname(),
        move |name| async move { get_stellar_host_detail(name).await },
    );

    // Fetch planets for this host
    let planets_resource = Resource::new(
        move || hostname(),
        move |name| async move { get_planets_for_host(name).await },
    );

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
            // Header
            <div class="relative overflow-hidden">
                <div class="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjAwIiBoZWlnaHQ9IjIwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZGVmcz48cGF0dGVybiBpZD0iZ3JpZCIgd2lkdGg9IjQwIiBoZWlnaHQ9IjQwIiBwYXR0ZXJuVW5pdHM9InVzZXJTcGFjZU9uVXNlIj48cGF0aCBkPSJNIDQwIDAgTCAwIDAgMCA0MCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSJyZ2JhKDI1NSwyNTUsMjU1LDAuMDUpIiBzdHJva2Utd2lkdGg9IjEiLz48L3BhdHRlcm4+PC9kZWZzPjxyZWN0IHdpZHRoPSIxMDAlIiBoZWlnaHQ9IjEwMCUiIGZpbGw9InVybCgjZ3JpZCkiLz48L3N2Zz4=')] opacity-20"></div>

                <div class="container mx-auto px-4 py-8 relative">
                    // Back button
                    <A
                        href="/stellarhosts"
                        attr:class="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors mb-6"
                    >
                        <span>"←"</span>
                        <span>"Back to Stellar Hosts"</span>
                    </A>

                    <div class="text-center space-y-2">
                        <div class="text-6xl mb-4">"⭐"</div>
                        <h1 class="text-4xl md:text-5xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-yellow-400 via-orange-400 to-red-400">
                            {hostname}
                        </h1>
                        <p class="text-lg text-gray-400">"Stellar Host System"</p>
                    </div>
                </div>
            </div>

            // Main content
            <div class="container mx-auto px-4 pb-16">
                <Suspense
                    fallback=move || {
                        view! {
                            <div class="flex flex-col justify-center items-center py-20">
                                <div class="relative">
                                    <div class="animate-spin rounded-full h-20 w-20 border-t-4 border-b-4 border-purple-500"></div>
                                    <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 text-4xl">
                                        "⭐"
                                    </div>
                                </div>
                                <span class="mt-6 text-lg text-gray-300 animate-pulse">"Loading stellar data..."</span>
                            </div>
                        }
                    }
                >
                    {move || {
                        let host_data = host_resource.get();
                        let planets_data = planets_resource.get();

                        match (host_data, planets_data) {
                            (Some(Ok(host)), Some(Ok(planets))) => {
                                view! {
                                    <div class="space-y-10">
                                        <StellarProperties host=host />
                                        <PlanetsSection planets=planets />
                                    </div>
                                }.into_any()
                            }
                            (Some(Err(e)), _) | (_, Some(Err(e))) => {
                                let error_msg = format!("Error: {}", e);
                                view! {
                                    <div class="max-w-2xl mx-auto mt-10 bg-red-900/50 border-2 border-red-500 text-red-100 px-6 py-4 rounded-xl backdrop-blur-sm">
                                        <div class="flex items-center gap-3">
                                            <span class="text-2xl">"⚠️"</span>
                                            <div>
                                                <h3 class="font-semibold text-lg">"Error Loading Data"</h3>
                                                <p class="text-sm text-red-200">{error_msg}</p>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            _ => {
                                view! { <div></div> }.into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn StellarProperties(host: StellarHostDetail) -> impl IntoView {
    // Group properties by category for better display
    let key_properties = vec![
        ("sy_dist", "Distance", "pc", "from-blue-600 to-cyan-500"),
        ("st_teff", "Temperature", "K", "from-orange-600 to-red-500"),
        ("st_mass", "Mass", "M☉", "from-purple-600 to-pink-500"),
        ("st_rad", "Radius", "R☉", "from-green-600 to-emerald-500"),
        ("sy_pnum", "Planets", "", "from-indigo-600 to-blue-500"),
        ("st_age", "Age", "Gyr", "from-amber-600 to-yellow-500"),
    ];

    view! {
        <div class="space-y-6">
            <h2 class="text-2xl font-bold text-white flex items-center gap-3">
                <span>"📊"</span>
                <span>"Key Properties"</span>
            </h2>

            // Key properties cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {key_properties.iter().map(|(key, label, unit, gradient)| {
                    let value = host.properties.get(*key).cloned();
                    let description = host.metadata.get(*key)
                        .and_then(|m| m.description.clone());

                    view! {
                        <PropertyCard
                            label=label.to_string()
                            value=value
                            unit=unit.to_string()
                            gradient=gradient.to_string()
                            description=description
                        />
                    }
                }).collect::<Vec<_>>()}
            </div>

            // All properties in collapsible section
            <details class="group">
                <summary class="cursor-pointer text-gray-400 hover:text-white transition-colors flex items-center gap-2">
                    <span class="group-open:rotate-90 transition-transform">"▶"</span>
                    <span>"All Properties ("{ host.properties.len() }" columns)"</span>
                </summary>
                <div class="mt-4 rounded-xl bg-slate-800/50 border border-slate-700 overflow-hidden">
                    <div class="max-h-96 overflow-y-auto">
                        <table class="w-full">
                            <thead class="bg-slate-900/50 sticky top-0">
                                <tr>
                                    <th class="px-4 py-3 text-left text-xs font-semibold text-gray-400 uppercase">"Property"</th>
                                    <th class="px-4 py-3 text-left text-xs font-semibold text-gray-400 uppercase">"Value"</th>
                                    <th class="px-4 py-3 text-left text-xs font-semibold text-gray-400 uppercase">"Description"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {host.properties.iter().map(|(key, value)| {
                                    let meta = host.metadata.get(key);
                                    let desc = meta.and_then(|m| m.description.clone()).unwrap_or_default();
                                    let desc_title = desc.clone();
                                    let unit = meta.and_then(|m| m.unit.clone()).unwrap_or_default();
                                    let formatted = format_value(value, &unit);

                                    view! {
                                        <tr class="border-t border-slate-700/50 hover:bg-slate-700/30">
                                            <td class="px-4 py-2 text-sm font-mono text-purple-400">{key.clone()}</td>
                                            <td class="px-4 py-2 text-sm text-white font-mono">{formatted}</td>
                                            <td class="px-4 py-2 text-xs text-gray-500 max-w-xs truncate" title=desc_title>{desc}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </details>
        </div>
    }
}

#[component]
fn PropertyCard(
    label: String,
    value: Option<Value>,
    unit: String,
    gradient: String,
    description: Option<String>,
) -> impl IntoView {
    let display_value = value
        .map(|v| format_value(&v, &unit))
        .unwrap_or_else(|| "—".to_string());

    view! {
        <div
            class="group relative overflow-hidden rounded-xl bg-slate-800/50 backdrop-blur-sm border border-slate-700 p-5 transition-all duration-300 hover:border-slate-500 hover:shadow-lg"
            title=description.unwrap_or_default()
        >
            <div class=format!("absolute inset-0 bg-gradient-to-br {} opacity-0 group-hover:opacity-10 transition-opacity duration-300", gradient)></div>
            <div class="relative z-10">
                <h3 class="text-sm font-medium text-gray-400 uppercase tracking-wider mb-2">
                    {label}
                </h3>
                <div class="text-2xl font-bold text-white font-mono">
                    {display_value}
                </div>
            </div>
        </div>
    }
}

#[component]
fn PlanetsSection(planets: HostPlanets) -> impl IntoView {
    let planet_count = planets.planets.len();

    view! {
        <div class="space-y-6">
            <h2 class="text-2xl font-bold text-white flex items-center gap-3">
                <span>"🪐"</span>
                <span>"Known Planets"</span>
                <span class="text-lg font-normal text-gray-400">
                    "(" {planet_count} " found)"
                </span>
            </h2>

            {if planets.planets.is_empty() {
                view! {
                    <div class="text-center py-12 text-gray-400 bg-slate-800/30 rounded-xl border border-slate-700">
                        <div class="text-4xl mb-4">"🌌"</div>
                        <p>"No confirmed planets found for this stellar host"</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        {planets.planets.iter().map(|planet| {
                            view! {
                                <PlanetCard planet=planet.clone() />
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </div>
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

    let disc_year = planet
        .get("disc_year")
        .map(|v| format_value(v, ""))
        .unwrap_or_else(|| "—".to_string());

    let orbital_period = planet
        .get("pl_orbper")
        .map(|v| format_value(v, "days"))
        .unwrap_or_else(|| "—".to_string());

    let radius = planet
        .get("pl_rade")
        .map(|v| format_value(v, "R⊕"))
        .unwrap_or_else(|| "—".to_string());

    let mass = planet
        .get("pl_bmasse")
        .map(|v| format_value(v, "M⊕"))
        .unwrap_or_else(|| "—".to_string());

    let eq_temp = planet
        .get("pl_eqt")
        .map(|v| format_value(v, "K"))
        .unwrap_or_else(|| "—".to_string());

    view! {
        <div class="rounded-xl bg-slate-800/50 backdrop-blur-sm border border-slate-700 p-5 hover:border-purple-500/50 transition-all duration-300 hover:shadow-lg hover:shadow-purple-500/10">
            <div class="flex items-start justify-between mb-4">
                <div>
                    <h3 class="text-lg font-bold text-white">{name}</h3>
                    <p class="text-sm text-gray-400">{discovery_method}</p>
                </div>
                <span class="text-3xl">"🪐"</span>
            </div>

            <div class="grid grid-cols-2 gap-3 text-sm">
                <div>
                    <span class="text-gray-500">"Discovered: "</span>
                    <span class="text-white font-mono">{disc_year}</span>
                </div>
                <div>
                    <span class="text-gray-500">"Period: "</span>
                    <span class="text-white font-mono">{orbital_period}</span>
                </div>
                <div>
                    <span class="text-gray-500">"Radius: "</span>
                    <span class="text-white font-mono">{radius}</span>
                </div>
                <div>
                    <span class="text-gray-500">"Mass: "</span>
                    <span class="text-white font-mono">{mass}</span>
                </div>
                <div class="col-span-2">
                    <span class="text-gray-500">"Eq. Temp: "</span>
                    <span class="text-white font-mono">{eq_temp}</span>
                </div>
            </div>
        </div>
    }
}

fn format_value(value: &Value, unit: &str) -> String {
    match value {
        Value::Null => "—".to_string(),
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            let formatted = if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 {
                    format!("{:.0}", f)
                } else if f.abs() < 0.01 || f.abs() >= 10000.0 {
                    format!("{:.2e}", f)
                } else {
                    format!("{:.2}", f)
                }
            } else if let Some(i) = n.as_i64() {
                i.to_string()
            } else {
                n.to_string()
            };

            if unit.is_empty() {
                formatted
            } else {
                format!("{} {}", formatted, unit)
            }
        }
        Value::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
        _ => value.to_string(),
    }
}
