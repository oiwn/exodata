use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_insight;
use exo_types::insights::planet_host_ratios::META;

#[component]
pub fn LargestPlanetToHostRatiosPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || META.slug.to_string(),
        move |slug| async move { get_insight(slug).await },
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
