use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

#[component]
pub fn Navbar() -> impl IntoView {
    let location = use_location();
    let pathname = move || location.pathname.get();

    let link_class =
        "px-4 py-2 rounded-lg transition-all duration-200 hover:bg-white/10";
    let active_class =
        "px-4 py-2 rounded-lg bg-white/20 text-purple-300 font-semibold";

    view! {
        <nav class="bg-slate-900/80 backdrop-blur-sm border-b border-purple-500/20 sticky top-0 z-50">
            <div class="container mx-auto px-4">
                <div class="flex justify-between items-center h-16">
                    <div class="flex items-center">
                        <span class="text-2xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-400">
                            "🪐 Exoplanets"
                        </span>
                    </div>

                    <div class="flex space-x-2 text-gray-300">
                        <A
                            href="/"
                            attr:class=move || {
                                if pathname() == "/" {
                                    active_class
                                } else {
                                    link_class
                                }
                            }
                        >
                            "Overview"
                        </A>

                        <A
                            href="/stellarhosts"
                            attr:class=move || {
                                if pathname() == "/stellarhosts" {
                                    active_class
                                } else {
                                    link_class
                                }
                            }
                        >
                            "Stellar Hosts"
                        </A>

                        <A
                            href="/exoplanets"
                            attr:class=move || {
                                if pathname() == "/exoplanets" {
                                    active_class
                                } else {
                                    link_class
                                }
                            }
                        >
                            "Exoplanets"
                        </A>

                        <A
                            href="/about"
                            attr:class=move || {
                                if pathname() == "/about" {
                                    active_class
                                } else {
                                    link_class
                                }
                            }
                        >
                            "About"
                        </A>
                    </div>
                </div>
            </div>
        </nav>
    }
}
