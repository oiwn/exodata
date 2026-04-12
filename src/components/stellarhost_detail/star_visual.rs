use leptos::prelude::*;

use super::star_color::StarVisualTokens;

#[component]
pub fn HostStarVisual(tokens: StarVisualTokens) -> impl IntoView {
    view! {
        <div class="star-visual">
            <div
                class="star-visual__halo--offset"
                style:background-color=tokens.halo_color.clone()
            ></div>
            <div class="star-visual__spark"></div>
            <div
                class="star-visual__halo"
                style:background-color=tokens.halo_color.clone()
            ></div>
            <div
                class="star-visual__core"
                style:background=tokens.core_gradient
                style:box-shadow=format!("0 0 80px {}", tokens.glow_color)
            ></div>
            <div
                class="star-visual__badge"
                style:background-color=tokens.badge_tint
            >
                <p class="star-visual__badge-label">"Approximate color"</p>
                <p class="star-visual__badge-value">"from effective temperature"</p>
            </div>
        </div>
    }
}
