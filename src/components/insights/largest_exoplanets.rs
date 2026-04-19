use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_largest_exoplanets_by_radius_insight;

#[component]
pub fn LargestExoplanetsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || (),
        move |_| async move { get_largest_exoplanets_by_radius_insight().await },
    );

    view! {
        <InsightListPageShell
            eyebrow="Planetary extremes"
            title="Largest exoplanets by radius"
            intro="Inflated giants and outsized confirmed worlds ranked by the current archive-backed radius field."
            description="The largest confirmed exoplanets in the catalog ordered by radius."
            resource=rows_resource
            empty_label="No planet rows available."
        />
    }
}
