use leptos::prelude::*;

use super::star_color::StarVisualTokens;

#[component]
pub fn HostStarVisual(tokens: StarVisualTokens) -> impl IntoView {
    view! {
        <div class="relative flex h-72 w-72 items-center justify-center">
            <div
                class="absolute right-[-5rem] top-[-4rem] h-56 w-56 rounded-full blur-3xl"
                style:background-color=tokens.halo_color.clone()
            ></div>
            <div class="absolute left-[18%] top-[30%] h-2 w-2 rounded-full bg-sky-200/60"></div>
            <div
                class="absolute inset-0 rounded-full blur-3xl"
                style:background-color=tokens.halo_color.clone()
            ></div>
            <div
                class="absolute h-56 w-56 rounded-full border border-amber-100/20"
                style:background=tokens.core_gradient
                style:box-shadow=format!("0 0 80px {}", tokens.glow_color)
            ></div>
            <div
                class="absolute bottom-6 right-4 rounded-full border border-white/10 px-4 py-2 text-right backdrop-blur"
                style:background-color=tokens.badge_tint
            >
                <p class="text-xs uppercase tracking-[0.18em] text-slate-400">"Approximate color"</p>
                <p class="text-sm text-white">"from effective temperature"</p>
            </div>
        </div>
    }
}
