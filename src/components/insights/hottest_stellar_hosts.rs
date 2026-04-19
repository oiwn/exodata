use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_hottest_stellar_hosts_insight;

#[component]
pub fn HottestStellarHostsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || (),
        move |_| async move { get_hottest_stellar_hosts_insight().await },
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
