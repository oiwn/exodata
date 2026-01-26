use leptos::prelude::*;

/// A loading overlay that appears on top of content while data is loading.
/// Uses fixed positioning so the spinner is always visible in the viewport.
#[component]
pub fn LoadingOverlay(
    /// Signal indicating whether loading is in progress
    loading: Signal<bool>,
) -> impl IntoView {
    view! {
        <Show when=move || loading.get()>
            // Overlay covers the table area with subtle dimming
            <div class="absolute inset-0 bg-slate-900/40 z-10 rounded-lg"></div>
            // Spinner is fixed to viewport center so it's always visible
            <div class="fixed inset-0 z-20 flex flex-col justify-center items-center pointer-events-none">
                <div class="bg-slate-800/90 rounded-xl px-6 py-4 flex flex-col items-center shadow-xl border border-slate-700">
                    <div class="relative">
                        <div class="animate-spin rounded-full h-12 w-12 border-t-4 border-b-4 border-purple-500"></div>
                        <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 text-2xl">
                            "🪐"
                        </div>
                    </div>
                    <span class="mt-3 text-sm text-gray-300">"Loading..."</span>
                </div>
            </div>
        </Show>
    }
}
