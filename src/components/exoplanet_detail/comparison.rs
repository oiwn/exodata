use leptos::prelude::*;

use super::format::{comparison_scale, format_number, median_numeric_value};
use crate::server::functions::ExoplanetDetail;

const EARTH_RADIUS_REARTH: f64 = 1.0;
const JUPITER_RADIUS_REARTH: f64 = 11.2;
const MAX_BODY_DIAMETER_REM: f64 = 10.5;

#[component]
pub fn ScaleComparisonSection(detail: ExoplanetDetail) -> impl IntoView {
    let radius = median_numeric_value(&detail.records, "pl_rade");

    if let Some(radius_rearth) = radius {
        let max_radius = EARTH_RADIUS_REARTH
            .max(radius_rearth)
            .max(JUPITER_RADIUS_REARTH);
        let earth_scale = comparison_scale(EARTH_RADIUS_REARTH, max_radius);
        let planet_scale = comparison_scale(radius_rearth, max_radius);
        let jupiter_scale = comparison_scale(JUPITER_RADIUS_REARTH, max_radius);

        view! {
            <section class="planet-detail-section">
                <div class="planet-detail-section__header">
                    <div>
                        <p class="planet-detail-section__eyebrow planet-detail-section__eyebrow--comparison">"Scale Comparison"</p>
                        <h2 class="planet-detail-section__title">"Radius against familiar reference worlds"</h2>
                    </div>
                    <p class="planet-detail-section__description planet-detail-section__description--comparison">
                        {format!("Current adopted radius: {} R⊕ • circles scaled linearly by radius", format_number(radius_rearth))}
                    </p>
                </div>

                <div class="planet-comparison">
                    <ScaleBody
                        label="Earth".to_string()
                        subtitle="1.0 R⊕ baseline".to_string()
                        scale=earth_scale
                        modifier="planet-comparison__body--earth"
                    />
                    <ScaleBody
                        label=detail.pl_name.clone()
                        subtitle=format!("{} R⊕", format_number(radius_rearth))
                        scale=planet_scale
                        modifier="planet-comparison__body--planet"
                    />
                    <ScaleBody
                        label="Jupiter".to_string()
                        subtitle="11.2 R⊕ reference".to_string()
                        scale=jupiter_scale
                        modifier="planet-comparison__body--jupiter"
                    />
                </div>
            </section>
        }
        .into_any()
    } else {
        view! { <div></div> }.into_any()
    }
}

#[component]
fn ScaleBody(
    label: String,
    subtitle: String,
    scale: f64,
    modifier: &'static str,
) -> impl IntoView {
    let diameter_rem = (MAX_BODY_DIAMETER_REM * scale).max(0.2);
    let size = format!("{diameter_rem:.3}rem");

    view! {
        <article class="planet-comparison__card">
            <div class="planet-comparison__body-wrap">
                <div
                    class=format!("planet-comparison__body {modifier}")
                    style:width=size.clone()
                    style:height=size
                ></div>
            </div>
            <p class="planet-comparison__label">{label}</p>
            <p class="planet-comparison__subtitle">{subtitle}</p>
        </article>
    }
}
