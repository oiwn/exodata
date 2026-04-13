use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_distinct_stellarhosts_page;

const COLUMNS: &str = "hostname,st_teff,st_mass,sy_dist,sy_pnum";

#[component]
pub fn HottestStellarHostsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || (),
        move |_| async move {
            get_distinct_stellarhosts_page(
                1,
                10,
                Some("st_teff".to_string()),
                Some("desc".to_string()),
                Some(COLUMNS.to_string()),
                None,
            )
            .await
        },
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
