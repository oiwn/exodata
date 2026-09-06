use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::lazy_route;

use crate::metadata_helpers::{SITE_URL, canonical_url, title_with_site};
use crate::{i18n::*, locale::localized_path};
// Import server function and types - #[server] macro handles client/server compilation
use crate::components::homepage_manual::HomepageManual;
use crate::server::functions::{DataStats, get_stats};
use crate::structured_data::{StructuredData, website_schema};

// --- Lazy Route ---

#[derive(Clone)]
pub struct OverviewLazy;

#[lazy_route]
impl LazyRoute for OverviewLazy {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <OverviewPage/> }.into_any()
    }
}

#[component]
pub fn OverviewPage() -> impl IntoView {
    let i18n = use_i18n();
    let locale = i18n.get_locale_untracked();
    let canonical_path = localized_path("/", locale);
    // Create a resource that calls the server function
    let stats_resource =
        Resource::new(move || (), move |_| async move { get_stats().await });

    view! {
        <Title text=title_with_site(t_string!(i18n, home.title))/>
        <Meta name="description" content=t_string!(i18n, home.description)/>
        <Link rel="canonical" href=canonical_url(&canonical_path)/>
        <Link rel="alternate" hreflang="en" href=format!("{SITE_URL}/")/>
        <Link rel="alternate" hreflang="zh-CN" href=format!("{SITE_URL}/zh-CN")/>
        <Link rel="alternate" hreflang="ja" href=format!("{SITE_URL}/ja")/>
        <Link rel="alternate" hreflang="x-default" href=format!("{SITE_URL}/")/>
        <StructuredData value=website_schema()/>
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
            // Header with cosmic background
            <div class="relative overflow-hidden">
                <div class="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjAwIiBoZWlnaHQ9IjIwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZGVmcz48cGF0dGVybiBpZD0iZ3JpZCIgd2lkdGg9IjQwIiBoZWlnaHQ9IjQwIiBwYXR0ZXJuVW5pdHM9InVzZXJTcGFjZU9uVXNlIj48cGF0aCBkPSJNIDQwIDAgTCAwIDAgMCA0MCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSJyZ2JhKDI1NSwyNTUsMjU1LDAuMDUpIiBzdHJva2Utd2lkdGg9IjEiLz48L3BhdHRlcm4+PC9kZWZzPjxyZWN0IHdpZHRoPSIxMDAlIiBoZWlnaHQ9IjEwMCUiIGZpbGw9InVybCgjZ3JpZCkiLz48L3N2Zz4=')] opacity-20"></div>

                <div class="container mx-auto px-4 py-16 relative">
                    <div class="text-center space-y-4">
                        <h1 class="text-5xl md:text-6xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 animate-pulse">
                            {t!(i18n, home.hero_title)}
                        </h1>
                        <a
                            href="#mcp-exoplanet-data"
                            class="block text-xl text-gray-300 max-w-2xl mx-auto transition-colors hover:text-white focus:outline-none focus:ring-2 focus:ring-purple-400 rounded-lg"
                        >
                            {t!(i18n, home.hero_subtitle)}
                        </a>
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
                                <span class="mt-6 text-lg text-gray-300 animate-pulse">
                                    {t!(i18n, home.loading)}
                                </span>
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
                                    <HomepageManual/>
                                </div>
                            }),
                            Err(err) => {
                                let error_msg = format!("{}: {err}", t_string!(i18n, home.error_loading));
                                leptos::either::Either::Right(view! {
                                    <div class="max-w-2xl mx-auto mt-10 bg-red-900/50 border-2 border-red-500 text-red-100 px-6 py-4 rounded-xl backdrop-blur-sm">
                                        <div class="flex items-center gap-3">
                                            <span class="text-2xl">"⚠️"</span>
                                            <div>
                                                <h3 class="font-semibold text-lg">{t!(i18n, home.connection_error)}</h3>
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
    let i18n = use_i18n();
    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <StatCard
                title=t_string!(i18n, home.stellar_systems)
                value=stats.stellarhosts_total.to_string()
                icon="⭐"
                subtitle=t_string!(i18n, home.host_stars_catalogued)
                gradient="from-blue-600 to-cyan-500"
            />
            <StatCard
                title=t_string!(i18n, home.exoplanets)
                value=stats.exoplanets_total.to_string()
                icon="🪐"
                subtitle=t_string!(i18n, home.distinct_planets)
                gradient="from-purple-600 to-pink-500"
            />
            <StatCard
                title=t_string!(i18n, home.average_temperature)
                value=format!("{:.0} K", stats.avg_stellar_temp)
                icon="🌡️"
                subtitle=t_string!(i18n, home.mean_temperature)
                gradient="from-orange-600 to-red-500"
            />
            <StatCard
                title=t_string!(i18n, home.average_distance)
                value=format!("{:.1} pc", stats.avg_stellar_distance)
                icon="📏"
                subtitle=t_string!(i18n, home.mean_distance)
                gradient="from-green-600 to-emerald-500"
            />
        </div>
    }
}

#[component]
fn DetailedStats(stats: DataStats) -> impl IntoView {
    let i18n = use_i18n();
    let locale = i18n.get_locale_untracked();
    let planet_size_categories =
        localized_overview_labels(stats.planet_size_categories, locale);
    let orbital_period_buckets =
        localized_overview_labels(stats.orbital_period_buckets, locale);
    let stellar_classes = stellar_class_labels(stats.stellar_classes, locale);
    let discovery_methods =
        localized_overview_labels(stats.discovery_methods, locale);
    let planet_temperature_bands =
        localized_overview_labels(stats.planet_temperature_bands, locale);
    let detection_sources =
        localized_overview_labels(stats.detection_sources, locale);
    view! {
        <div class="space-y-6">
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">
                <StatSection
                    title=t_string!(i18n, home.planet_classifications)
                    subtitle=t_string!(i18n, home.planet_classifications_subtitle)
                    icon="🌍"
                    items=planet_size_categories
                />
                <StatSection
                    title=t_string!(i18n, home.orbital_periods)
                    subtitle=t_string!(i18n, home.orbital_periods_subtitle)
                    icon="🌀"
                    items=orbital_period_buckets
                />
            </div>
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">
                <StatSection
                    title=t_string!(i18n, home.planet_mass_distribution)
                    subtitle=t_string!(i18n, home.planet_mass_distribution_subtitle)
                    icon="⚖️"
                    items=stats.planet_mass_bands
                />
                <StatSection
                    title=t_string!(i18n, home.stellar_classes)
                    subtitle=t_string!(i18n, home.stellar_classes_subtitle)
                    icon="✨"
                    items=stellar_classes
                />
            </div>
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">
                <StatSection
                    title=t_string!(i18n, home.discovery_methods)
                    subtitle=t_string!(i18n, home.discovery_methods_subtitle)
                    icon="🔭"
                    items=discovery_methods
                />
                <StatSection
                    title=t_string!(i18n, home.discovery_years)
                    subtitle=t_string!(i18n, home.discovery_years_subtitle)
                    icon="📅"
                    items=stats.discovery_years
                />
            </div>
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">
                <StatSection
                    title=t_string!(i18n, home.temperature_bands)
                    subtitle=t_string!(i18n, home.temperature_bands_subtitle)
                    icon="🌡️"
                    items=planet_temperature_bands
                />
                <StatSection
                    title=t_string!(i18n, home.detection_sources)
                    subtitle=t_string!(i18n, home.detection_sources_subtitle)
                    icon="🛰️"
                    items=detection_sources
                />
            </div>
        </div>
    }
}

fn localized_overview_labels(
    items: Vec<(String, usize)>,
    locale: Locale,
) -> Vec<(String, usize)> {
    items
        .into_iter()
        .map(|(label, count)| {
            let localized = match label.as_str() {
                "Sub-Earth (< 1 R⊕)" => {
                    Some(td_string!(locale, home.planet_size_sub_earth))
                }
                "Earth-like (1-1.5 R⊕)" => {
                    Some(td_string!(locale, home.planet_size_earth_like))
                }
                "Super-Earth (1.5-2.5 R⊕)" => {
                    Some(td_string!(locale, home.planet_size_super_earth))
                }
                "Neptune-like (2.5-4 R⊕)" => {
                    Some(td_string!(locale, home.planet_size_neptune_like))
                }
                "Jupiter-like (> 4 R⊕)" => {
                    Some(td_string!(locale, home.planet_size_jupiter_like))
                }
                "< 1 day" => {
                    Some(td_string!(locale, home.orbital_period_under_one_day))
                }
                "1-10 days" => {
                    Some(td_string!(locale, home.orbital_period_one_to_ten_days))
                }
                "10-100 days" => Some(td_string!(
                    locale,
                    home.orbital_period_ten_to_hundred_days
                )),
                "100-1000 days" => Some(td_string!(
                    locale,
                    home.orbital_period_hundred_to_thousand_days
                )),
                "> 1000 days" => Some(td_string!(
                    locale,
                    home.orbital_period_over_thousand_days
                )),
                "Ultra-hot (> 1500 K)" => {
                    Some(td_string!(locale, home.temperature_ultra_hot))
                }
                "Very hot (1000-1500 K)" => {
                    Some(td_string!(locale, home.temperature_very_hot))
                }
                "Hot (700-1000 K)" => {
                    Some(td_string!(locale, home.temperature_hot))
                }
                "Warm (500-700 K)" => {
                    Some(td_string!(locale, home.temperature_warm))
                }
                "Mild (350-500 K)" => {
                    Some(td_string!(locale, home.temperature_mild))
                }
                "Temperate (200-350 K)" => {
                    Some(td_string!(locale, home.temperature_temperate))
                }
                "Cold (< 200 K)" => {
                    Some(td_string!(locale, home.temperature_cold))
                }
                "Astrometry" => {
                    Some(td_string!(locale, home.discovery_method_astrometry))
                }
                "Disk Kinematics" => Some(td_string!(
                    locale,
                    home.discovery_method_disk_kinematics
                )),
                "Eclipse Timing Variations" => {
                    Some(td_string!(locale, home.discovery_method_eclipse_timing))
                }
                "Imaging" => {
                    Some(td_string!(locale, home.discovery_method_imaging))
                }
                "Microlensing" => {
                    Some(td_string!(locale, home.discovery_method_microlensing))
                }
                "Orbital Brightness Modulation" => Some(td_string!(
                    locale,
                    home.discovery_method_orbital_brightness
                )),
                "Pulsar Timing" => {
                    Some(td_string!(locale, home.discovery_method_pulsar_timing))
                }
                "Pulsation Timing Variations" => Some(td_string!(
                    locale,
                    home.discovery_method_pulsation_timing
                )),
                "Radial Velocity" => Some(td_string!(
                    locale,
                    home.discovery_method_radial_velocity
                )),
                "Transit" => {
                    Some(td_string!(locale, home.discovery_method_transit))
                }
                "Transit Timing Variations" => {
                    Some(td_string!(locale, home.discovery_method_transit_timing))
                }
                "Other" => Some(td_string!(locale, home.other)),
                _ => None,
            };

            (localized.map(str::to_string).unwrap_or(label), count)
        })
        .collect()
}

fn stellar_class_labels(
    classes: Vec<(String, usize)>,
    locale: Locale,
) -> Vec<(String, usize)> {
    classes
        .into_iter()
        .map(|(class, count)| {
            let description = match class.as_str() {
                "O" => Some(td_string!(locale, home.stellar_class_o)),
                "B" => Some(td_string!(locale, home.stellar_class_b)),
                "A" => Some(td_string!(locale, home.stellar_class_a)),
                "F" => Some(td_string!(locale, home.stellar_class_f)),
                "G" => Some(td_string!(locale, home.stellar_class_g)),
                "K" => Some(td_string!(locale, home.stellar_class_k)),
                "M" => Some(td_string!(locale, home.stellar_class_m)),
                _ => None,
            };
            let label = description
                .map(|description| format!("{class} — {description}"))
                .unwrap_or(class);

            (label, count)
        })
        .collect()
}

#[component]
fn StatCard(
    title: &'static str,
    value: String,
    icon: &'static str,
    subtitle: &'static str,
    gradient: &'static str,
) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <div class="group relative overflow-hidden rounded-2xl bg-slate-800/50 backdrop-blur-sm border border-slate-700 p-6 transition-all duration-300 hover:scale-105 hover:border-slate-500 hover:shadow-2xl hover:shadow-purple-500/20">
            // Gradient overlay
            <div class=format!("absolute inset-0 bg-gradient-to-br {} opacity-0 group-hover:opacity-10 transition-opacity duration-300", gradient)></div>

            <div class="relative z-10">
                <div class="flex items-start justify-between mb-4">
                    <span class="text-5xl filter drop-shadow-lg">{icon}</span>
                    <div class=format!("px-3 py-1 rounded-full bg-gradient-to-r {} text-white text-xs font-bold", gradient)>
                        {t!(i18n, home.live)}
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

#[cfg(test)]
mod tests {
    use super::{localized_overview_labels, stellar_class_labels};
    use crate::i18n::Locale;

    #[test]
    fn stellar_class_labels_localize_standard_classes_and_preserve_counts() {
        let classes = vec![("G".to_string(), 3), ("M".to_string(), 2)];

        assert_eq!(
            stellar_class_labels(classes.clone(), Locale::en),
            vec![
                ("G — yellow dwarf (Sun-like)".to_string(), 3),
                ("M — red dwarf".to_string(), 2),
            ]
        );
        assert_eq!(
            stellar_class_labels(classes.clone(), Locale::zh_CN),
            vec![
                ("G — 黄矮星（类似太阳）".to_string(), 3),
                ("M — 红矮星".to_string(), 2),
            ]
        );
        assert_eq!(
            stellar_class_labels(classes, Locale::ja),
            vec![
                ("G — 黄色矮星（太陽型）".to_string(), 3),
                ("M — 赤色矮星".to_string(), 2),
            ]
        );
    }

    #[test]
    fn stellar_class_labels_preserve_unknown_classes() {
        assert_eq!(
            stellar_class_labels(vec![("W".to_string(), 1)], Locale::en),
            vec![("W".to_string(), 1)]
        );
    }

    #[test]
    fn overview_labels_localize_categories_and_preserve_unknown_values() {
        let labels = vec![
            ("Super-Earth (1.5-2.5 R⊕)".to_string(), 6),
            ("1-10 days".to_string(), 5),
            ("Transit".to_string(), 4),
            ("Radial Velocity".to_string(), 3),
            ("Ultra-hot (> 1500 K)".to_string(), 2),
            ("Kepler".to_string(), 1),
        ];

        assert_eq!(
            localized_overview_labels(labels.clone(), Locale::zh_CN),
            vec![
                ("超级地球（1.5-2.5 R⊕）".to_string(), 6),
                ("1-10 天".to_string(), 5),
                ("凌日法".to_string(), 4),
                ("径向速度法".to_string(), 3),
                ("超高温（> 1500 K）".to_string(), 2),
                ("Kepler".to_string(), 1),
            ]
        );
        assert_eq!(
            localized_overview_labels(labels, Locale::ja),
            vec![
                ("スーパーアース（1.5-2.5 R⊕）".to_string(), 6),
                ("1-10日".to_string(), 5),
                ("トランジット法".to_string(), 4),
                ("視線速度法".to_string(), 3),
                ("超高温（> 1500 K）".to_string(), 2),
                ("Kepler".to_string(), 1),
            ]
        );
    }
}
