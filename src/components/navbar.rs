use leptos::prelude::*;
use leptos::web_sys::KeyboardEvent;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

use crate::i18n::*;
use crate::locale::{
    locale_from_path, localized_path, localized_url, strip_locale_prefix,
};

#[component]
pub fn Navbar() -> impl IntoView {
    let location = use_location();
    let i18n = use_i18n();
    let pathname = move || location.pathname.get();
    let current_path = move || strip_locale_prefix(&pathname()).to_string();
    let current_locale = move || locale_from_path(&pathname());
    let is_stellarhosts = move || current_path().starts_with("/stellarhosts");
    let is_exoplanets = move || current_path().starts_with("/exoplanets");
    let is_insights = move || current_path().starts_with("/insights");
    let is_docs = move || {
        current_path().starts_with("/docs")
            || current_path().starts_with("/about")
    };

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
    let icon_link_class = "flex items-center justify-center w-10 h-10 rounded-lg text-gray-300 transition-all duration-200 hover:bg-white/10 hover:text-white";

    // Mobile link styles
    let mobile_link_class = "text-xl font-medium text-gray-300 hover:text-purple-400 py-3 px-6 rounded-lg hover:bg-white/5 w-full text-center transition-colors";
    let mobile_active_class = "text-xl font-medium text-purple-300 bg-white/10 py-3 px-6 rounded-lg w-full text-center";

    view! {
        <nav class="bg-slate-900/80 backdrop-blur-sm border-b border-purple-500/20 sticky top-0 z-50">
            <div class="container mx-auto px-4">
                <div class="flex justify-between items-center h-16">
                    <A href=move || localized_path("/", current_locale()) attr:class="flex items-center">
                        <span class="text-xl md:text-2xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-400">
                            "🪐 Exoplanets"
                        </span>
                    </A>

                    // Desktop Navigation (hidden on mobile)
                    <div class="hidden md:flex space-x-2 text-gray-300">
                        <A
                            href=move || localized_path("/stellarhosts", current_locale())
                            attr:class=move || {
                                if is_stellarhosts() {
                                    active_class
                                } else {
                                    link_class
                                }
                            }
                        >
                            {t!(i18n, nav.stellar_hosts)}
                        </A>

                        <A
                            href=move || localized_path("/exoplanets", current_locale())
                            attr:class=move || {
                                if is_exoplanets() {
                                    active_class
                                } else {
                                    link_class
                                }
                            }
                        >
                            {t!(i18n, nav.exoplanets)}
                        </A>

                        <A
                            href=move || localized_path("/insights", current_locale())
                            attr:class=move || {
                                if is_insights() {
                                    active_class
                                } else {
                                    link_class
                                }
                            }
                        >
                            {t!(i18n, nav.insights)}
                        </A>

                        <A
                            href=move || localized_path("/docs", current_locale())
                            attr:class=move || {
                                if is_docs() {
                                    active_class
                                } else {
                                    link_class
                                }
                            }
                        >
                            {t!(i18n, nav.docs)}
                        </A>

                        <a
                            href="/swagger-ui"
                            class=link_class
                            target="_blank"
                        >
                            {t!(i18n, nav.api)}
                        </a>

                        <a
                            href="https://github.com/oiwn/exodata"
                            class=icon_link_class
                            target="_blank"
                            rel="noopener noreferrer"
                            aria-label=move || t_string!(i18n, nav.github_repository)
                            title=move || t_string!(i18n, nav.github_repository)
                        >
                            {github_icon()}
                        </a>
                        <LanguageSwitcher/>
                    </div>

                    // Hamburger Button (visible on mobile only)
                    <button
                        class="md:hidden flex items-center justify-center w-10 h-10 rounded-lg hover:bg-white/10 transition-colors"
                        aria-label=move || t_string!(i18n, nav.open_menu)
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
                    aria-label=move || t_string!(i18n, nav.mobile_navigation)
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
                            aria-label=move || t_string!(i18n, nav.close_menu)
                            on:click=move |_| set_mobile_menu_open.set(false)
                        >
                            {close_icon()}
                        </button>
                    </div>

                    // Navigation Links
                    <div class="flex-1 flex flex-col items-center justify-center space-y-4 px-6">
                        <A
                            href=move || localized_path("/", current_locale())
                            attr:class=move || {
                                if current_path() == "/" { mobile_active_class } else { mobile_link_class }
                            }
                        >
                            {t!(i18n, nav.overview)}
                        </A>
                        <A
                            href=move || localized_path("/stellarhosts", current_locale())
                            attr:class=move || {
                                if is_stellarhosts() { mobile_active_class } else { mobile_link_class }
                            }
                        >
                            {t!(i18n, nav.stellar_hosts)}
                        </A>
                        <A
                            href=move || localized_path("/exoplanets", current_locale())
                            attr:class=move || {
                                if is_exoplanets() { mobile_active_class } else { mobile_link_class }
                            }
                        >
                            {t!(i18n, nav.exoplanets)}
                        </A>
                        <A
                            href=move || localized_path("/insights", current_locale())
                            attr:class=move || {
                                if is_insights() { mobile_active_class } else { mobile_link_class }
                            }
                        >
                            {t!(i18n, nav.insights)}
                        </A>
                        <A
                            href=move || localized_path("/docs", current_locale())
                            attr:class=move || {
                                if is_docs() { mobile_active_class } else { mobile_link_class }
                            }
                        >
                            {t!(i18n, nav.docs)}
                        </A>

                        <a
                            href="/swagger-ui"
                            class=mobile_link_class
                            target="_blank"
                        >
                            {t!(i18n, nav.api)}
                        </a>
                        <LanguageSwitcher/>

                        <a
                            href="https://github.com/oiwn/exodata"
                            class=mobile_link_class
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            <span class="inline-flex items-center gap-3">
                                {github_icon()}
                                <span>"GitHub"</span>
                            </span>
                        </a>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn LanguageSwitcher() -> impl IntoView {
    let location = use_location();
    let i18n = use_i18n();
    let current_locale = move || locale_from_path(&location.pathname.get());
    let href_for = move |locale| {
        localized_url(
            &location.pathname.get(),
            &location.search.get(),
            &location.hash.get(),
            locale,
        )
    };
    let option_class = move |locale| {
        if current_locale() == locale {
            "rounded-md bg-purple-500/25 px-2.5 py-1.5 text-xs font-semibold text-purple-200"
        } else {
            "rounded-md px-2.5 py-1.5 text-xs font-semibold text-slate-300 transition-colors hover:bg-white/10 hover:text-white"
        }
    };

    view! {
        <div
            class="flex items-center gap-0.5 rounded-lg border border-slate-700 bg-slate-950/60 p-1"
            role="group"
            aria-label=move || t_string!(i18n, nav.language)
        >
            <A
                href=move || href_for(Locale::en)
                attr:class=move || option_class(Locale::en)
                attr:lang="en"
                attr:title="English"
            >
                "EN"
            </A>
            <A
                href=move || href_for(Locale::zh_CN)
                attr:class=move || option_class(Locale::zh_CN)
                attr:lang="zh-CN"
                attr:title="简体中文"
            >
                "中文"
            </A>
            <A
                href=move || href_for(Locale::ja)
                attr:class=move || option_class(Locale::ja)
                attr:lang="ja"
                attr:title="日本語"
            >
                "日本語"
            </A>
        </div>
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

fn github_icon() -> impl IntoView {
    view! {
        <svg
            class="w-5 h-5"
            viewBox="0 0 24 24"
            fill="currentColor"
            aria-hidden="true"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path d="M12 0.5C5.37 0.5 0 5.87 0 12.5C0 17.8 3.44 22.29 8.21 23.88C8.81 23.99 9.03 23.63 9.03 23.32C9.03 23.04 9.02 22.09 9.01 20.83C5.67 21.56 4.97 19.42 4.97 19.42C4.42 18.02 3.63 17.65 3.63 17.65C2.55 16.91 3.71 16.93 3.71 16.93C4.91 17.01 5.54 18.17 5.54 18.17C6.6 20 8.32 19.47 9 19.16C9.11 18.39 9.41 17.86 9.74 17.56C7.08 17.25 4.29 16.23 4.29 11.67C4.29 10.37 4.75 9.31 5.52 8.47C5.4 8.17 5 6.95 5.64 5.31C5.64 5.31 6.65 4.99 8.96 6.55C9.93 6.28 10.97 6.14 12 6.14C13.03 6.14 14.07 6.28 15.04 6.55C17.35 4.99 18.36 5.31 18.36 5.31C19 6.95 18.6 8.17 18.48 8.47C19.25 9.31 19.71 10.37 19.71 11.67C19.71 16.24 16.91 17.25 14.24 17.55C14.66 17.91 15.03 18.62 15.03 19.72C15.03 21.3 15.02 22.86 15.02 23.32C15.02 23.63 15.24 24 15.85 23.88C20.62 22.29 24 17.8 24 12.5C24 5.87 18.63 0.5 12 0.5Z"/>
        </svg>
    }
}
