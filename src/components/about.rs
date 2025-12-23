use leptos::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 py-16">
            <div class="container mx-auto px-4 max-w-4xl">
                <div class="bg-slate-800/50 backdrop-blur-sm rounded-xl p-8 border border-purple-500/20 shadow-2xl">
                    <h1 class="text-4xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 via-purple-400 to-pink-400 mb-6">
                        "About This Project"
                    </h1>

                    <div class="space-y-6 text-gray-300 text-lg leading-relaxed">
                        <p>
                            "Welcome to the Exoplanets Catalog, a comprehensive database exploring the wonders of confirmed exoplanets and their stellar hosts across the cosmos."
                        </p>

                        <div class="space-y-4">
                            <h2 class="text-2xl font-semibold text-purple-300">"🌌 What We Offer"</h2>
                            <ul class="list-disc list-inside space-y-2 ml-4">
                                <li>"Detailed information on thousands of confirmed exoplanets"</li>
                                <li>"Comprehensive stellar host data including temperature, mass, and distance"</li>
                                <li>"Interactive data visualizations and statistics"</li>
                                <li>"Sortable and paginated tables for easy exploration"</li>
                                <li>"Real-time data analysis and aggregation"</li>
                            </ul>
                        </div>

                        <div class="space-y-4">
                            <h2 class="text-2xl font-semibold text-purple-300">"🔭 Technology Stack"</h2>
                            <p>
                                "Built with modern web technologies using Rust and Leptos framework, this catalog provides a fast and responsive experience. Data is stored in Parquet format for efficient querying and analysis."
                            </p>
                        </div>

                        <div class="space-y-4">
                            <h2 class="text-2xl font-semibold text-purple-300">"🪐 Our Mission"</h2>
                            <p>
                                "To make exoplanet data accessible and engaging for researchers, students, and space enthusiasts. By providing an intuitive interface to explore the vast catalog of discovered worlds, we hope to inspire curiosity about the universe beyond our solar system."
                            </p>
                        </div>

                        <div class="pt-6 border-t border-purple-500/20">
                            <p class="text-sm text-gray-400">
                                "Data sourced from astronomical surveys and exoplanet research databases."
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
