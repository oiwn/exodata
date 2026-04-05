use leptos::prelude::*;
use leptos::serde_json::Value;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use leptos_router::lazy_route;

use crate::metadata_helpers::{
    canonical_url, decode_path_segment, encode_path_segment,
    exoplanet_detail_description, exoplanet_detail_title, title_with_site,
};
use crate::server::functions::{ExoplanetDetail, get_exoplanet_detail};
use crate::structured_data::{StructuredData, exoplanet_dataset_schema};

// --- Lazy Route ---

#[derive(Clone)]
pub struct ExoplanetDetailLazy;

#[lazy_route]
impl LazyRoute for ExoplanetDetailLazy {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <ExoplanetDetailPage/> }.into_any()
    }
}

// --- Component ---

#[component]
pub fn ExoplanetDetailPage() -> impl IntoView {
    let params = use_params_map();
    let pl_name = Memo::new(move |_| {
        let raw = params.read().get("pl_name").unwrap_or_default();
        decode_path_segment(&raw)
    });

    let fallback_title = Signal::derive(move || {
        title_with_site(&format!("{} Exoplanet", pl_name.get()))
    });
    let fallback_description = Signal::derive(move || {
        format!(
            "Explore measurements and source records for the exoplanet {}.",
            pl_name.get()
        )
    });
    let canonical_href = Signal::derive(move || {
        canonical_url(&format!(
            "/exoplanets/{}",
            encode_path_segment(&pl_name.get())
        ))
    });

    let detail_resource = Resource::new(
        move || pl_name.get(),
        move |name| async move { get_exoplanet_detail(name).await },
    );

    view! {
        <Title text=move || {
            detail_resource
                .get()
                .and_then(|result| result.ok().map(|detail| exoplanet_detail_title(&detail)))
                .unwrap_or_else(|| fallback_title.get())
        }/>
        <Meta name="description" content=move || {
            detail_resource
                .get()
                .and_then(|result| result.ok().map(|detail| exoplanet_detail_description(&detail)))
                .unwrap_or_else(|| fallback_description.get())
        }/>
        <Link rel="canonical" href=canonical_href.get()/>
        {move || {
            detail_resource.get().and_then(|result| {
                result.ok().map(|detail| {
                    view! { <StructuredData value=exoplanet_dataset_schema(&detail)/> }
                })
            })
        }}
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
            <div class="relative overflow-hidden">
                <div class="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjAwIiBoZWlnaHQ9IjIwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZGVmcz48cGF0dGVybiBpZD0iZ3JpZCIgd2lkdGg9IjQwIiBoZWlnaHQ9IjQwIiBwYXR0ZXJuVW5pdHM9InVzZXJTcGFjZU9uVXNlIj48cGF0aCBkPSJNIDQwIDAgTCAwIDAgMCA0MCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSJyZ2JhKDI1NSwyNTUsMjU1LDAuMDUpIiBzdHJva2Utd2lkdGg9IjEiLz48L3BhdHRlcm4+PC9kZWZzPjxyZWN0IHdpZHRoPSIxMDAlIiBoZWlnaHQ9IjEwMCUiIGZpbGw9InVybCgjZ3JpZCkiLz48L3N2Zz4=')] opacity-20"></div>

                <div class="container mx-auto px-4 py-8 relative">
                    <A
                        href="/exoplanets"
                        attr:class="inline-flex items-center gap-2 text-gray-400 hover:text-white transition-colors mb-6"
                    >
                        <span>"←"</span>
                        <span>"Back to Exoplanets"</span>
                    </A>

                    <div class="text-center space-y-2">
                        <div class="text-6xl mb-4">"🪐"</div>
                        <h1 class="text-4xl md:text-5xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400">
                            {move || pl_name.get()}
                        </h1>
                        <p class="text-lg text-gray-400">"Exoplanet Records"</p>
                    </div>
                </div>
            </div>

            <div class="container mx-auto px-4 pb-16">
                <Suspense
                    fallback=move || {
                        view! {
                            <div class="flex flex-col justify-center items-center py-20">
                                <div class="relative">
                                    <div class="animate-spin rounded-full h-20 w-20 border-t-4 border-b-4 border-purple-500"></div>
                                    <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 text-4xl">
                                        "🪐"
                                    </div>
                                </div>
                                <span class="mt-6 text-lg text-gray-300 animate-pulse">"Loading exoplanet data..."</span>
                            </div>
                        }
                    }
                >
                    {move || {
                        detail_resource.get().map(|result| match result {
                            Ok(detail) => view! {
                                <div class="space-y-10">
                                    <PlanetSummary detail=detail.clone() />
                                    <PlanetRecords detail=detail />
                                </div>
                            }.into_any(),
                            Err(err) => {
                                let error_msg = format!("Error: {}", err);
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
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn PlanetSummary(detail: ExoplanetDetail) -> impl IntoView {
    let record = detail.records.first().cloned().unwrap_or(Value::Null);
    let metadata = detail.metadata.clone();

    let key_properties = [
        ("hostname", "Host Star", "", "from-blue-600 to-cyan-500"),
        (
            "discoverymethod",
            "Discovery",
            "",
            "from-indigo-600 to-blue-500",
        ),
        ("disc_year", "Year", "", "from-purple-600 to-pink-500"),
        (
            "pl_orbper",
            "Period",
            "days",
            "from-amber-600 to-yellow-500",
        ),
        ("pl_rade", "Radius", "R⊕", "from-green-600 to-emerald-500"),
        ("pl_bmasse", "Mass", "M⊕", "from-orange-600 to-red-500"),
        ("pl_eqt", "Eq. Temp", "K", "from-slate-600 to-gray-500"),
    ];

    view! {
        <div class="space-y-6">
            <h2 class="text-2xl font-bold text-white flex items-center gap-3">
                <span>"📊"</span>
                <span>"Key Properties"</span>
            </h2>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {key_properties.iter().map(|(key, label, fallback_unit, gradient)| {
                    let value = record.get(*key).cloned();
                    let meta = metadata.get(*key);
                    let unit = meta
                        .and_then(|m| m.unit.clone())
                        .unwrap_or_else(|| fallback_unit.to_string());
                    let description = meta.and_then(|m| m.description.clone());

                    view! {
                        <PropertyCard
                            label=label.to_string()
                            value=value
                            unit=unit
                            gradient=gradient.to_string()
                            description=description
                        />
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn PlanetRecords(detail: ExoplanetDetail) -> impl IntoView {
    let total = detail.records.len();
    let metadata = detail.metadata.clone();

    view! {
        <div class="space-y-6">
            <h2 class="text-2xl font-bold text-white flex items-center gap-3">
                <span>"🧾"</span>
                <span>"Records"</span>
                <span class="text-lg font-normal text-gray-400">
                    "(" {total} " rows)"
                </span>
            </h2>

            <div class="space-y-4">
                {detail.records.into_iter().enumerate().map(|(idx, record)| {
                    let metadata = metadata.clone();
                    view! {
                        <RecordCard index=idx record=record metadata=metadata />
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn RecordCard(
    index: usize,
    record: Value,
    metadata: std::collections::HashMap<
        String,
        crate::server::functions::ColumnMetadata,
    >,
) -> impl IntoView {
    let method = record
        .get("discoverymethod")
        .and_then(|v| v.as_str())
        .unwrap_or("—")
        .to_string();
    let year = record
        .get("disc_year")
        .map(|v| format_value(v, ""))
        .unwrap_or_else(|| "—".to_string());
    let facility = record
        .get("disc_facility")
        .and_then(|v| v.as_str())
        .unwrap_or("—")
        .to_string();

    view! {
        <div class="rounded-xl bg-slate-800/50 backdrop-blur-sm border border-slate-700 p-5 hover:border-purple-500/40 transition-all duration-300">
            <div class="flex items-start justify-between gap-4">
                <div>
                    <h3 class="text-lg font-bold text-white">
                        {format!("Record {}", index + 1)}
                    </h3>
                    <p class="text-sm text-gray-400">
                        {format!("{} • {} • {}", method, year, facility)}
                    </p>
                </div>
                <span class="text-2xl">"🪐"</span>
            </div>

            <details class="group mt-4">
                <summary class="cursor-pointer text-gray-400 hover:text-white transition-colors flex items-center gap-2">
                    <span class="group-open:rotate-90 transition-transform">"▶"</span>
                    <span>"All Properties"</span>
                </summary>
                <div class="mt-4 rounded-xl bg-slate-900/40 border border-slate-700 overflow-hidden">
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
                                {record.as_object().map(|map| {
                                    map.iter().map(|(key, value)| {
                                        let meta = metadata.get(key);
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
                                    }).collect::<Vec<_>>()
                                }).unwrap_or_default()}
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
