use leptos::prelude::*;

// --- Content Constants ---

const INTRO: &str = "A searchable catalog of confirmed exoplanets built on NASA Exoplanet Archive data. \
Query planetary masses, orbital periods, stellar hosts, and discovery methods across 5,000+ worlds.";

const DATA_SOURCE_TITLE: &str = "Data Source";
const DATA_SOURCE_TEXT: &str = "Raw data comes from the NASA Exoplanet Archive's Planetary Systems Composite table. \
We fetch the VOTable export, parse it, and store in Parquet format for fast columnar queries.";
const DATA_SOURCE_EXAMPLE: &str = "Example: pl_bmasse (planet mass in Earth masses), pl_orbper (orbital period in days), \
st_teff (stellar effective temperature in Kelvin)";

const TECH_TITLE: &str = "How It Works";
const TECH_ITEMS: &[&str] = &[
    "Backend: Rust + Axum serving a REST API with raw SQL query support",
    "Frontend: Leptos (Rust → WASM) with server-side rendering",
    "Storage: Parquet files queried via DuckDB in-process",
    "API: GET /rest/query?sql=SELECT... for arbitrary queries",
];

const API_TITLE: &str = "REST API";
const API_TEXT: &str = "The catalog exposes a SQL query endpoint. You can run arbitrary SELECT statements against the dataset.";
const API_EXAMPLES: &[(&str, &str)] = &[
    ("Find hot Jupiters", "SELECT pl_name, pl_bmasse, pl_eqt FROM exoplanets WHERE pl_bmasse > 100 AND pl_eqt > 1000 LIMIT 10"),
    ("Count by discovery method", "SELECT discoverymethod, COUNT(*) as cnt FROM exoplanets GROUP BY discoverymethod ORDER BY cnt DESC"),
    ("Nearest stars with planets", "SELECT pl_name, sy_dist FROM exoplanets WHERE sy_dist IS NOT NULL ORDER BY sy_dist LIMIT 20"),
];

const FOOTER_TEXT: &str = "Data: NASA Exoplanet Archive. Code: Rust, Leptos, Axum, DuckDB.";

// --- Component ---

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
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

                        // API Examples
                        <Section title=API_TITLE>
                            <p class="mb-4">{API_TEXT}</p>
                            <div class="space-y-4">
                                {API_EXAMPLES.iter().map(|(label, sql)| view! {
                                    <div class="bg-slate-900/50 p-4 rounded">
                                        <p class="text-sm text-gray-400 mb-2">{*label}</p>
                                        <code class="block text-sm text-green-400 font-mono break-all">
                                            {*sql}
                                        </code>
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
