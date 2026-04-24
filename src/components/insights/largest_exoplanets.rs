use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_insight;
use exo_types::insights::largest_exoplanets::META;

#[component]
pub fn LargestExoplanetsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || META.slug.to_string(),
        move |slug| async move { get_insight(slug).await },
    );

    view! {
        <InsightListPageShell
            eyebrow="Planetary extremes"
            title="Largest exoplanets by radius"
            intro="Inflated giants and outsized confirmed worlds ranked by the current archive-default radius field."
            description="The largest confirmed exoplanets in the catalog ordered by archive-default radius."
            resource=rows_resource
            empty_label="No planet rows available."
        />
    }
}
