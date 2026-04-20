use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_most_distant_exoplanets_insight;

#[component]
pub fn MostDistantExoplanetsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || (),
        move |_| async move { get_most_distant_exoplanets_insight().await },
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
