use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_insight;
use exo_types::insights::nearest_stellar_hosts::META;

#[component]
pub fn NearestStellarHostsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || META.slug.to_string(),
        move |slug| async move { get_insight(slug).await },
    );

    view! {
        <InsightListPageShell
            eyebrow="Distance"
            title="Nearest stellar hosts"
            intro="Nearby planet-hosting stars ranked by archive distance from Earth."
            description="The nearest stellar hosts with confirmed planets, ordered by distance from Earth."
            resource=rows_resource
            empty_label="No stellar-host distance rows available."
        />
    }
}
