use leptos::prelude::*;
use leptos_router::components::A;

#[cfg(feature = "ssr")]
use crate::server::functions::DataStats;

#[cfg(feature = "ssr")]
use crate::server::functions::get_stats;

// DataStats struct for client-side (must match server definition)
#[cfg(not(feature = "ssr"))]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DataStats {
    pub stellarhosts_total: usize,
    pub exoplanets_total: usize,
    pub avg_stellar_temp: f64,
    pub avg_stellar_distance: f64,
    pub discovery_methods: Vec<(String, usize)>,
    pub planet_size_categories: Vec<(String, usize)>,
}

#[cfg(not(feature = "ssr"))]
#[leptos::server]
pub async fn get_stats() -> Result<DataStats, leptos::server_fn::ServerFnError> {
    // This will be replaced by actual implementation on the server
    unreachable!()
}

#[component]
pub fn OverviewPage() -> impl IntoView {
    // Create a resource that calls the server function
    let stats_resource =
        Resource::new(move || (), move |_| async move { get_stats().await });

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
            // Header with cosmic background
            <div class="relative overflow-hidden">
                <div class="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjAwIiBoZWlnaHQ9IjIwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZGVmcz48cGF0dGVybiBpZD0iZ3JpZCIgd2lkdGg9IjQwIiBoZWlnaHQ9IjQwIiBwYXR0ZXJuVW5pdHM9InVzZXJTcGFjZU9uVXNlIj48cGF0aCBkPSJNIDQwIDAgTCAwIDAgMCA0MCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSJyZ2JhKDI1NSwyNTUsMjU1LDAuMDUpIiBzdHJva2Utd2lkdGg9IjEiLz48L3BhdHRlcm4+PC9kZWZzPjxyZWN0IHdpZHRoPSIxMDAlIiBoZWlnaHQ9IjEwMCUiIGZpbGw9InVybCgjZ3JpZCkiLz48L3N2Zz4=')] opacity-20"></div>

                <div class="container mx-auto px-4 py-16 relative">
                    <div class="text-center space-y-4">
                        <h1 class="text-5xl md:text-6xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 animate-pulse">
                            "🌌 Exoplanet Archive"
                        </h1>
                        <p class="text-xl text-gray-300 max-w-2xl mx-auto">
                            "Exploring the cosmos: A comprehensive catalog of confirmed exoplanets and their host stars"
                        </p>
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
                                        "🪐"
                                    </div>
                                </div>
                                <span class="mt-6 text-lg text-gray-300 animate-pulse">"Loading cosmic data..."</span>
                            </div>
                        }
                    }
                >
                    {move || {
                        stats_resource.get().map(|result| match result {
                            Ok(stats) => leptos::either::Either::Left(view! {
                                <div class="space-y-10">
                                    <StatsOverview stats=stats.clone()/>
                                    <DetailedStats stats=stats/>

                                    // Link to stellar hosts table
                                    <div class="text-center">
                                        <A
                                            href="/stellarhosts"
                                            attr:class="inline-flex items-center gap-2 px-8 py-4 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-500 hover:to-purple-500 text-white font-semibold rounded-xl shadow-lg hover:shadow-xl transition-all duration-300 transform hover:scale-105"
                                        >
                                            <span>"⭐"</span>
                                            <span>"Browse Stellar Hosts Catalog"</span>
                                            <span>"→"</span>
                                        </A>
                                    </div>
                                </div>
                            }),
                            Err(err) => {
                                let error_msg = format!("Error loading data: {}", err);
                                leptos::either::Either::Right(view! {
                                    <div class="max-w-2xl mx-auto mt-10 bg-red-900/50 border-2 border-red-500 text-red-100 px-6 py-4 rounded-xl backdrop-blur-sm">
                                        <div class="flex items-center gap-3">
                                            <span class="text-2xl">"⚠️"</span>
                                            <div>
                                                <h3 class="font-semibold text-lg">"Connection Error"</h3>
                                                <p class="text-sm text-red-200">{error_msg}</p>
                                            </div>
                                        </div>
                                    </div>
                                })
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn StatsOverview(stats: DataStats) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <StatCard
                title="Stellar Systems"
                value=stats.stellarhosts_total.to_string()
                icon="⭐"
                subtitle="Host stars catalogued"
                gradient="from-blue-600 to-cyan-500"
            />
            <StatCard
                title="Exoplanets"
                value=stats.exoplanets_total.to_string()
                icon="🪐"
                subtitle="Confirmed discoveries"
                gradient="from-purple-600 to-pink-500"
            />
            <StatCard
                title="Avg Stellar Temp"
                value=format!("{:.0} K", stats.avg_stellar_temp)
                icon="🌡️"
                subtitle="Mean effective temperature"
                gradient="from-orange-600 to-red-500"
            />
            <StatCard
                title="Avg Distance"
                value=format!("{:.1} pc", stats.avg_stellar_distance)
                icon="📏"
                subtitle="Mean system distance"
                gradient="from-green-600 to-emerald-500"
            />
        </div>
    }
}

#[component]
fn DetailedStats(stats: DataStats) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <StatSection
                title="Discovery Methods"
                subtitle="How we find exoplanets"
                icon="🔭"
                items=stats.discovery_methods
            />
            <StatSection
                title="Planet Classifications"
                subtitle="Size distribution by Earth radii"
                icon="🌍"
                items=stats.planet_size_categories
            />
        </div>
    }
}

#[component]
fn StatCard(
    title: &'static str,
    value: String,
    icon: &'static str,
    subtitle: &'static str,
    gradient: &'static str,
) -> impl IntoView {
    view! {
        <div class="group relative overflow-hidden rounded-2xl bg-slate-800/50 backdrop-blur-sm border border-slate-700 p-6 transition-all duration-300 hover:scale-105 hover:border-slate-500 hover:shadow-2xl hover:shadow-purple-500/20">
            // Gradient overlay
            <div class=format!("absolute inset-0 bg-gradient-to-br {} opacity-0 group-hover:opacity-10 transition-opacity duration-300", gradient)></div>

            <div class="relative z-10">
                <div class="flex items-start justify-between mb-4">
                    <span class="text-5xl filter drop-shadow-lg">{icon}</span>
                    <div class=format!("px-3 py-1 rounded-full bg-gradient-to-r {} text-white text-xs font-bold", gradient)>
                        "LIVE"
                    </div>
                </div>

                <h3 class="text-sm font-medium text-gray-400 uppercase tracking-wider mb-2">
                    {title}
                </h3>

                <div class="text-3xl font-bold text-white mb-2 font-mono">
                    {value}
                </div>

                <p class="text-xs text-gray-500">
                    {subtitle}
                </p>
            </div>
        </div>
    }
}

#[component]
fn StatSection(
    title: &'static str,
    subtitle: &'static str,
    icon: &'static str,
    items: Vec<(String, usize)>,
) -> impl IntoView {
    // Calculate total for percentages
    let total: usize = items.iter().map(|(_, count)| count).sum();

    view! {
        <div class="rounded-2xl bg-slate-800/50 backdrop-blur-sm border border-slate-700 p-8 hover:border-slate-600 transition-all duration-300">
            <div class="flex items-center gap-3 mb-6">
                <span class="text-4xl">{icon}</span>
                <div>
                    <h3 class="text-2xl font-bold text-white">{title}</h3>
                    <p class="text-sm text-gray-400">{subtitle}</p>
                </div>
            </div>

            <div class="space-y-3">
                {items.into_iter().enumerate().map(|(idx, (name, count))| {
                    let percentage = if total > 0 {
                        count as f64 / total as f64 * 100.0
                    } else {
                        0.0
                    };

                    let bar_color = match idx % 6 {
                        0 => "bg-blue-500",
                        1 => "bg-purple-500",
                        2 => "bg-pink-500",
                        3 => "bg-orange-500",
                        4 => "bg-green-500",
                        _ => "bg-cyan-500",
                    };

                    view! {
                        <div class="group">
                            <div class="flex justify-between items-center mb-2">
                                <span class="text-sm font-medium text-gray-300 group-hover:text-white transition-colors">
                                    {name}
                                </span>
                                <div class="flex items-center gap-3">
                                    <span class="text-xs text-gray-500 font-mono">
                                        {format!("{:.1}%", percentage)}
                                    </span>
                                    <span class="font-bold text-white font-mono min-w-[4rem] text-right">
                                        {count.to_string()}
                                    </span>
                                </div>
                            </div>
                            <div class="h-2 bg-slate-700 rounded-full overflow-hidden">
                                <div
                                    class=format!("{} h-full rounded-full transition-all duration-500 ease-out", bar_color)
                                    style=format!("width: {}%", percentage)
                                ></div>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
