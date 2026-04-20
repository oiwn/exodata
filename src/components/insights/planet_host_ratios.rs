use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_largest_planet_to_host_ratios_insight;

#[component]
pub fn LargestPlanetToHostRatiosPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || (),
        move |_| async move { get_largest_planet_to_host_ratios_insight().await },
    );

    view! {
        <InsightListPageShell
            eyebrow="Relationships"
            title="Largest planet-to-host radius ratios"
            intro="Extreme size-ratio systems ranked by planet radius divided by host-star radius, using archive-default planet rows."
            description="Planetary systems with the largest planet-to-host radius ratios."
            resource=rows_resource
            empty_label="No planet and host radius rows available."
        />
    }
}
