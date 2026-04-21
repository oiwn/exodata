use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_insight;
use exo_types::insights::smallest_exoplanets::META;

#[component]
pub fn SmallestExoplanetsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || META.slug.to_string(),
        move |slug| async move { get_insight(slug).await },
    );

    view! {
        <InsightListPageShell
            eyebrow="Planetary extremes"
            title="Smallest exoplanets by radius"
            intro="A compact ranked view of the smallest confirmed exoplanets using the current archive-default radius field."
            description="The smallest confirmed exoplanets in the catalog ordered by archive-default radius."
            resource=rows_resource
            empty_label="No planet rows available."
        />
    }
}
