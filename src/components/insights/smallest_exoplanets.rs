use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_smallest_exoplanets_by_radius_insight;

#[component]
pub fn SmallestExoplanetsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || (),
        move |_| async move { get_smallest_exoplanets_by_radius_insight().await },
    );

    view! {
        <InsightListPageShell
            eyebrow="Planetary extremes"
            title="Smallest exoplanets by radius"
            intro="A compact ranked view of the smallest confirmed exoplanets using the current archive-backed radius field."
            description="The smallest confirmed exoplanets in the catalog ordered by radius."
            resource=rows_resource
            empty_label="No planet rows available."
        />
    }
}
