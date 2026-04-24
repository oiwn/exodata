use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_insight;
use exo_types::insights::crowded_systems::META;

#[component]
pub fn SystemsWithMostPlanetsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || META.slug.to_string(),
        move |slug| async move { get_insight(slug).await },
    );

    view! {
        <InsightListPageShell
            eyebrow="System architecture"
            title="Planetary systems with most planets"
            intro="A compact ranked list of planetary systems using the archive-backed system name and confirmed planet count."
            description="The confirmed planetary systems with the largest planet counts in the catalog."
            resource=rows_resource
            empty_label="No planetary systems available."
        />
    }
}
