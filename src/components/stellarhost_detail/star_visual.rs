use leptos::prelude::*;

#[component]
pub fn HostStarVisual() -> impl IntoView {
    view! {
        <div class="relative flex h-72 w-72 items-center justify-center">
            <div class="absolute right-[-5rem] top-[-4rem] h-56 w-56 rounded-full bg-amber-300/20 blur-3xl"></div>
            <div class="absolute left-[18%] top-[30%] h-2 w-2 rounded-full bg-sky-200/60"></div>
            <div class="absolute inset-0 rounded-full bg-amber-300/20 blur-3xl"></div>
            <div class="absolute h-56 w-56 rounded-full border border-amber-100/20 bg-[radial-gradient(circle_at_30%_30%,_rgba(255,251,235,0.95),_rgba(251,191,36,0.9)_35%,_rgba(180,83,9,0.85)_75%,_rgba(120,53,15,0.95))] shadow-[0_0_80px_rgba(251,191,36,0.35)]"></div>
            <div class="absolute bottom-6 right-4 rounded-full border border-white/10 bg-slate-950/60 px-4 py-2 text-right backdrop-blur">
                <p class="text-xs uppercase tracking-[0.18em] text-slate-400">"Approximate color"</p>
                <p class="text-sm text-white">"from effective temperature"</p>
            </div>
        </div>
    }
}
