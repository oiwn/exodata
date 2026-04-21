use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_insight;
use exo_types::insights::equal_star_planet_pairs::META;

#[component]
pub fn MostEqualStarPlanetPairsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || META.slug.to_string(),
        move |slug| async move { get_insight(slug).await },
    );

    view! {
        <InsightListPageShell
            eyebrow="Relationships"
            title="Most equal star-planet pairs"
            intro="Systems ranked by how closely the planet radius matches the host-star radius after converting both to the same unit."
            description="Planetary systems where planet and host-star radii are unusually close in relative scale."
            resource=rows_resource
            empty_label="No planet and host radius rows available."
        />
    }
}
