use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_nearest_stellar_hosts_insight;

#[component]
pub fn NearestStellarHostsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || (),
        move |_| async move { get_nearest_stellar_hosts_insight().await },
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
