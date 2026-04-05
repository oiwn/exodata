use leptos::prelude::*;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_TIMESTAMP: &str = match option_env!("BUILD_TIMESTAMP") {
    Some(value) => value,
    None => "unknown",
};

#[component]
pub fn VersionFooter() -> impl IntoView {
    view! {
        <footer class="w-full border-t border-slate-800 bg-slate-950">
            <div class="mx-auto flex max-w-7xl flex-col items-center justify-center gap-2 px-4 py-3 sm:flex-row sm:gap-4">
                <span class="inline-flex items-center gap-2 rounded-full border border-cyan-400/30 bg-slate-900 px-3 py-1 text-sm font-medium text-slate-100">
                    <span class="text-slate-400">"Version"</span>
                    <span class="font-semibold text-cyan-300">{APP_VERSION}</span>
                </span>
                <span class="inline-flex items-center gap-2 rounded-full border border-slate-700 bg-slate-900 px-3 py-1 text-sm font-medium text-slate-100">
                    <span class="text-slate-400">"Updated"</span>
                    <span class="font-semibold text-slate-200">{BUILD_TIMESTAMP}</span>
                </span>
                <a
                    href="https://www.imscraping.ninja"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="text-sm font-medium text-slate-200 hover:text-cyan-300"
                >
                    "Developed by imscraping.ninja"
                </a>
            </div>
        </footer>
    }
}
