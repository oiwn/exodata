use leptos::prelude::*;
use leptos::web_sys::KeyboardEvent;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

#[component]
pub fn Navbar() -> impl IntoView {
    let location = use_location();
    let pathname = move || location.pathname.get();

    // Mobile menu state
    let (mobile_menu_open, set_mobile_menu_open) = signal(false);

    // Close menu on route change
    Effect::new(move |_| {
        let _ = pathname();
        set_mobile_menu_open.set(false);
    });

    // Desktop link styles
    let link_class =
        "px-4 py-2 rounded-lg transition-all duration-200 hover:bg-white/10";
    let active_class =
        "px-4 py-2 rounded-lg bg-white/20 text-purple-300 font-semibold";

    // Mobile link styles
    let mobile_link_class = "text-xl font-medium text-gray-300 hover:text-purple-400 py-3 px-6 rounded-lg hover:bg-white/5 w-full text-center transition-colors";
    let mobile_active_class = "text-xl font-medium text-purple-300 bg-white/10 py-3 px-6 rounded-lg w-full text-center";

    view! {
        <nav class="bg-slate-900/80 backdrop-blur-sm border-b border-purple-500/20 sticky top-0 z-50">
            <div class="container mx-auto px-4">
                <div class="flex justify-between items-center h-16">
                    <div class="flex items-center">
                        <span class="text-xl md:text-2xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-400">
                            "🪐 Exoplanets"
                        </span>
                    </div>

                    // Desktop Navigation (hidden on mobile)
                    <div class="hidden md:flex space-x-2 text-gray-300">
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

                        <a
                            href="/swagger-ui"
                            class=link_class
                            target="_blank"
                        >
                            "API"
                        </a>
                    </div>

                    // Hamburger Button (visible on mobile only)
                    <button
                        class="md:hidden flex items-center justify-center w-10 h-10 rounded-lg hover:bg-white/10 transition-colors"
                        aria-label="Open navigation menu"
                        aria-expanded=move || mobile_menu_open.get()
                        aria-controls="mobile-menu"
                        on:click=move |_| set_mobile_menu_open.set(true)
                    >
                        {hamburger_icon()}
                    </button>
                </div>
            </div>
        </nav>

        // Mobile Menu Overlay
        <Show when=move || mobile_menu_open.get()>
            <div class="fixed inset-0 z-50 md:hidden">
                // Backdrop
                <div
                    class="absolute inset-0 bg-slate-900/95 backdrop-blur-md"
                    on:click=move |_| set_mobile_menu_open.set(false)
                />

                // Menu Content
                <div
                    id="mobile-menu"
                    class="relative z-10 flex flex-col h-full"
                    role="navigation"
                    aria-label="Mobile navigation"
                    on:keydown=move |e: KeyboardEvent| {
                        if e.key() == "Escape" {
                            set_mobile_menu_open.set(false);
                        }
                    }
                >
                    // Header with close button
                    <div class="flex items-center justify-between px-4 h-16 border-b border-purple-500/20">
                        <span class="text-xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-400">
                            "🪐 Exoplanets"
                        </span>
                        <button
                            class="flex items-center justify-center w-10 h-10 rounded-lg hover:bg-white/10 transition-colors"
                            aria-label="Close navigation menu"
                            on:click=move |_| set_mobile_menu_open.set(false)
                        >
                            {close_icon()}
                        </button>
                    </div>

                    // Navigation Links
                    <div class="flex-1 flex flex-col items-center justify-center space-y-4 px-6">
                        <A
                            href="/"
                            attr:class=move || {
                                if pathname() == "/" { mobile_active_class } else { mobile_link_class }
                            }
                        >
                            "Overview"
                        </A>
                        <A
                            href="/stellarhosts"
                            attr:class=move || {
                                if pathname() == "/stellarhosts" { mobile_active_class } else { mobile_link_class }
                            }
                        >
                            "Stellar Hosts"
                        </A>
                        <A
                            href="/exoplanets"
                            attr:class=move || {
                                if pathname() == "/exoplanets" { mobile_active_class } else { mobile_link_class }
                            }
                        >
                            "Exoplanets"
                        </A>
                        <A
                            href="/about"
                            attr:class=move || {
                                if pathname() == "/about" { mobile_active_class } else { mobile_link_class }
                            }
                        >
                            "About"
                        </A>

                        <a
                            href="/swagger-ui"
                            class=mobile_link_class
                            target="_blank"
                        >
                            "API"
                        </a>
                    </div>
                </div>
            </div>
        </Show>
    }
}

fn hamburger_icon() -> impl IntoView {
    view! {
        <svg
            class="w-6 h-6 text-gray-300"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 6h16M4 12h16M4 18h16"
            />
        </svg>
    }
}

fn close_icon() -> impl IntoView {
    view! {
        <svg
            class="w-6 h-6 text-gray-300"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M6 18L18 6M6 6l12 12"
            />
        </svg>
    }
}
