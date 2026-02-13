use leptos::prelude::*;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[component]
pub fn VersionFooter() -> impl IntoView {
    view! {
        <footer class="w-full border-t border-slate-700/50 bg-slate-900/60 backdrop-blur-sm">
            <div class="container mx-auto px-4 py-2 text-center text-xs text-gray-400">
                {format!("version {}", APP_VERSION)}
            </div>
        </footer>
    }
}
