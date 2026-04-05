use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::components::A;
use leptos_router::hooks::use_location;
use leptos_router::lazy_route;

use crate::metadata_helpers::{about_description, about_title, canonical_url};
// --- Content Constants ---

const INTRO: &str = "A searchable catalog of confirmed exoplanets and their host stars built on NASA Exoplanet Archive data. \
Filter, sort, and run read-only SQL queries across thousands of worlds.";

const DATA_SOURCE_TITLE: &str = "Data Source";
const DATA_SOURCE_TEXT: &str = "Raw data comes from NASA Exoplanet Archive VOTable exports (stellar hosts and planetary systems). \
We convert to Parquet and load into in-memory Polars DataFrames for fast columnar queries.";
const DATA_SOURCE_EXAMPLE: &str = "Example: pl_bmasse (planet mass in Earth masses), pl_orbper (orbital period in days), \
st_teff (stellar effective temperature in Kelvin), sy_dist (distance in parsecs)";

const TECH_TITLE: &str = "How It Works";
const TECH_ITEMS: &[&str] = &[
    "Backend: Rust + Axum serving a REST API with raw SQL query support",
    "Frontend: Leptos (Rust → WASM) with server-side rendering",
    "Storage: Parquet files loaded into Polars DataFrames in-process",
    "API: /rest/stellarhosts, /rest/exoplanets, /rest/query (SELECT only)",
];

const EXPLORE_TITLE: &str = "Explore";
const EXPLORE_NOTE: &str = "Table links keep any current query string so filters and column choices can persist.";

const API_TITLE: &str = "REST API";
const API_TEXT: &str = "The catalog exposes a SQL query endpoint. Queries are read-only and accept SELECT statements.";
const API_EXAMPLES: &[(&str, &str, &str)] = &[
    (
        "Find hot Jupiters",
        "SELECT pl_name, pl_bmasse, pl_eqt FROM exoplanets WHERE pl_bmasse > 100 AND pl_eqt > 1000 LIMIT 10",
        "/rest/query?sql=SELECT%20pl_name,%20pl_bmasse,%20pl_eqt%20FROM%20exoplanets%20WHERE%20pl_bmasse%20%3E%20100%20AND%20pl_eqt%20%3E%201000%20LIMIT%2010",
    ),
    (
        "Count by discovery method",
        "SELECT discoverymethod, COUNT(*) as cnt FROM exoplanets GROUP BY discoverymethod ORDER BY cnt DESC",
        "/rest/query?sql=SELECT%20discoverymethod,%20COUNT(*)%20as%20cnt%20FROM%20exoplanets%20GROUP%20BY%20discoverymethod%20ORDER%20BY%20cnt%20DESC",
    ),
    (
        "Nearest stars with planets",
        "SELECT pl_name, sy_dist FROM exoplanets WHERE sy_dist IS NOT NULL ORDER BY sy_dist LIMIT 20",
        "/rest/query?sql=SELECT%20pl_name,%20sy_dist%20FROM%20exoplanets%20WHERE%20sy_dist%20IS%20NOT%20NULL%20ORDER%20BY%20sy_dist%20LIMIT%2020",
    ),
];

const FOOTER_TEXT: &str =
    "Data: NASA Exoplanet Archive. Code: Rust, Leptos, Axum, Polars.";

// --- Lazy Route ---

#[derive(Clone)]
pub struct AboutLazy;

#[lazy_route]
impl LazyRoute for AboutLazy {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <AboutPage/> }.into_any()
    }
}

// --- Component ---

#[component]
pub fn AboutPage() -> impl IntoView {
    let location = use_location();
    let stellarhosts_href = move || {
        let search = location.search.get();
        if search.is_empty() {
            "/stellarhosts?filter=Gliese".to_string()
        } else {
            format!("/stellarhosts{}", search)
        }
    };
    let exoplanets_href = move || {
        let search = location.search.get();
        if search.is_empty() {
            "/exoplanets?filter=Kepler".to_string()
        } else {
            format!("/exoplanets{}", search)
        }
    };

    view! {
        <Title text=about_title()/>
        <Meta name="description" content=about_description()/>
        <Link rel="canonical" href=canonical_url("/about")/>
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 py-16">
            <div class="container mx-auto px-4 max-w-4xl">
                <div class="bg-slate-800/50 backdrop-blur-sm rounded-xl p-8 border border-purple-500/20 shadow-2xl">
                    <h1 class="text-4xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 mb-6">
                        "Exoplanets Catalog"
                    </h1>

                    <div class="space-y-8 text-gray-300 text-lg leading-relaxed">
                        // Intro
                        <p>{INTRO}</p>

                        // Data Source
                        <Section title=DATA_SOURCE_TITLE>
                            <p>{DATA_SOURCE_TEXT}</p>
                            <code class="block mt-3 text-sm text-purple-300 bg-slate-900/50 p-3 rounded font-mono">
                                {DATA_SOURCE_EXAMPLE}
                            </code>
                        </Section>

                        // Tech Stack
                        <Section title=TECH_TITLE>
                            <ul class="space-y-2 font-mono text-base">
                                {TECH_ITEMS.iter().map(|item| view! {
                                    <li class="flex items-start">
                                        <span class="text-purple-400 mr-2">"→"</span>
                                        {*item}
                                    </li>
                                }).collect_view()}
                            </ul>
                        </Section>

                        // Explore Links
                        <Section title=EXPLORE_TITLE>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <A
                                    href=move || stellarhosts_href()
                                    attr:class="block rounded-lg border border-purple-500/20 bg-slate-900/50 p-4 hover:border-purple-400 hover:bg-slate-900/70 transition-colors"
                                >
                                    <p class="text-sm text-gray-400">"Stellar Hosts"</p>
                                    <p class="text-lg text-purple-200 font-semibold">"Browse filtered table"</p>
                                </A>
                                <A
                                    href=move || exoplanets_href()
                                    attr:class="block rounded-lg border border-purple-500/20 bg-slate-900/50 p-4 hover:border-purple-400 hover:bg-slate-900/70 transition-colors"
                                >
                                    <p class="text-sm text-gray-400">"Exoplanets"</p>
                                    <p class="text-lg text-purple-200 font-semibold">"Browse filtered table"</p>
                                </A>
                                <a
                                    href="/swagger-ui"
                                    class="block rounded-lg border border-purple-500/20 bg-slate-900/50 p-4 hover:border-purple-400 hover:bg-slate-900/70 transition-colors"
                                    target="_blank"
                                >
                                    <p class="text-sm text-gray-400">"API Docs"</p>
                                    <p class="text-lg text-purple-200 font-semibold">"Swagger UI"</p>
                                </a>
                                <a
                                    href="/rest/stellarhosts/schema"
                                    class="block rounded-lg border border-purple-500/20 bg-slate-900/50 p-4 hover:border-purple-400 hover:bg-slate-900/70 transition-colors"
                                    target="_blank"
                                >
                                    <p class="text-sm text-gray-400">"Schema"</p>
                                    <p class="text-lg text-purple-200 font-semibold">"Stellar hosts columns"</p>
                                </a>
                            </div>
                            <p class="text-sm text-gray-500 mt-3">{EXPLORE_NOTE}</p>
                        </Section>

                        // API Examples
                        <Section title=API_TITLE>
                            <p class="mb-4">{API_TEXT}</p>
                            <div class="space-y-4">
                                {API_EXAMPLES.iter().map(|(label, sql, href)| view! {
                                    <div class="bg-slate-900/50 p-4 rounded">
                                        <p class="text-sm text-gray-400 mb-2">{*label}</p>
                                        <code class="block text-sm text-green-400 font-mono break-all">
                                            {*sql}
                                        </code>
                                        <a
                                            href=*href
                                            class="inline-block mt-3 text-sm text-purple-300 hover:text-purple-200 underline"
                                            target="_blank"
                                        >
                                            "Run in API"
                                        </a>
                                    </div>
                                }).collect_view()}
                            </div>
                        </Section>

                        // Footer
                        <div class="pt-6 border-t border-purple-500/20">
                            <p class="text-sm text-gray-500 font-mono">{FOOTER_TEXT}</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Section(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="space-y-3">
            <h2 class="text-xl font-semibold text-purple-300 font-mono">{title}</h2>
            {children()}
        </div>
    }
}
