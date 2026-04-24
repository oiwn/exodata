use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_insight;
use exo_types::insights::distant_exoplanets::META;

#[component]
pub fn MostDistantExoplanetsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || META.slug.to_string(),
        move |slug| async move { get_insight(slug).await },
    );

    view! {
        <InsightListPageShell
            eyebrow="Orbital architecture"
            title="Most distant exoplanets from their stars"
            intro="Confirmed planets ranked by archive-default semi-major axis, using orbital distance rather than inferred temperature."
            description="The most distant confirmed exoplanets from their host stars, ordered by archive-default semi-major axis."
            resource=rows_resource
            empty_label="No orbital-distance rows available."
        />
    }
}
