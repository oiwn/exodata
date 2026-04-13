use leptos::prelude::*;

use super::format::format_number;
use super::star_color::star_visual_tokens;
use crate::server::functions::StellarHostDetail;

const SUN_RADIUS_RSUN: f64 = 1.0;
const SUN_TEFF_K: f64 = 5772.0;
const MAX_BODY_DIAMETER_REM: f64 = 10.5;

#[component]
pub fn StarScaleComparisonSection(host: StellarHostDetail) -> impl IntoView {
    let Some(radius_summary) = host.star.radius.clone() else {
        return view! { <div></div> }.into_any();
    };

    let host_radius = radius_summary.value;
    let host_scale =
        comparison_scale(host_radius, host_radius.max(SUN_RADIUS_RSUN));
    let sun_scale =
        comparison_scale(SUN_RADIUS_RSUN, host_radius.max(SUN_RADIUS_RSUN));
    let host_tokens =
        star_visual_tokens(host.star.teff.as_ref().map(|teff| teff.value));
    let sun_tokens = star_visual_tokens(Some(SUN_TEFF_K));

    view! {
        <section class="host-detail-section">
            <div class="host-detail-section__header">
                <div>
                    <p class="host-detail-section__eyebrow host-detail-section__eyebrow--comparison">"Scale Comparison"</p>
                    <h2 class="host-detail-section__title">"Radius against the Sun"</h2>
                </div>
                <p class="host-detail-section__description host-detail-section__description--comparison">
                    {format!("Current adopted radius: {} R☉ • circles scaled linearly by radius", format_number(host_radius))}
                </p>
            </div>

            <div class="host-comparison">
                <ScaleStar
                    label="Sun".to_string()
                    subtitle="1.0 R☉ baseline".to_string()
                    scale=sun_scale
                    core_gradient=sun_tokens.core_gradient
                    glow_color=sun_tokens.glow_color
                />
                <ScaleStar
                    label=host.hostname
                    subtitle=format!("{} R☉", format_number(host_radius))
                    scale=host_scale
                    core_gradient=host_tokens.core_gradient
                    glow_color=host_tokens.glow_color
                />
            </div>
        </section>
    }
    .into_any()
}

#[component]
fn ScaleStar(
    label: String,
    subtitle: String,
    scale: f64,
    core_gradient: String,
    glow_color: String,
) -> impl IntoView {
    let diameter_rem = (MAX_BODY_DIAMETER_REM * scale).max(0.25);
    let size = format!("{diameter_rem:.3}rem");

    view! {
        <article class="host-comparison__card">
            <div class="host-comparison__body-wrap">
                <div
                    class="host-comparison__body"
                    style:width=size.clone()
                    style:height=size
                    style:background=core_gradient
                    style:box-shadow=format!("0 0 80px {glow_color}")
                ></div>
            </div>
            <p class="host-comparison__label">{label}</p>
            <p class="host-comparison__subtitle">{subtitle}</p>
        </article>
    }
}

fn comparison_scale(radius_rsun: f64, max_radius_rsun: f64) -> f64 {
    if max_radius_rsun <= 0.0 {
        0.0
    } else {
        radius_rsun / max_radius_rsun
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_scale_is_linear_for_smaller_and_larger_stars() {
        assert_eq!(comparison_scale(0.5, 1.0), 0.5);
        assert_eq!(comparison_scale(1.0, 2.0), 0.5);
        assert_eq!(comparison_scale(1.7, 1.7), 1.0);
    }

    #[test]
    fn comparison_scale_handles_invalid_max_radius() {
        assert_eq!(comparison_scale(1.0, 0.0), 0.0);
    }
}
