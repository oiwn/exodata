use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_distinct_stellarhosts_page;

const COLUMNS: &str = "hostname,sy_pnum,sy_dist,st_teff,st_mass";

#[component]
pub fn SystemsWithMostPlanetsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || (),
        move |_| async move {
            get_distinct_stellarhosts_page(
                1,
                10,
                Some("sy_pnum".to_string()),
                Some("desc".to_string()),
                Some(COLUMNS.to_string()),
                None,
            )
            .await
        },
    );

    view! {
        <InsightListPageShell
            eyebrow="System architecture"
            title="Systems with most planets"
            intro="A compact ranked list of host systems with the largest confirmed planet counts, presented without the main table-page controls."
            description="The confirmed stellar host systems with the largest planet counts in the catalog."
            resource=rows_resource
            empty_label="No stellar host rows available."
        />
    }
}
