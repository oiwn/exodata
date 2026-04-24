use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_insight;
use exo_types::insights::hottest_stellar_hosts::META;

#[component]
pub fn HottestStellarHostsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || META.slug.to_string(),
        move |slug| async move { get_insight(slug).await },
    );

    view! {
        <InsightListPageShell
            eyebrow="Stellar extremes"
            title="Hottest stellar hosts"
            intro="A focused list of confirmed host stars with the highest effective temperatures among systems currently in the catalog."
            description="The hottest stellar hosts in the catalog ordered by effective temperature."
            resource=rows_resource
            empty_label="No stellar host rows available."
        />
    }
}
