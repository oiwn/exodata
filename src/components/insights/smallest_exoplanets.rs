use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_distinct_exoplanets_page;

const COLUMNS: &str = "pl_name,hostname,pl_rade,pl_bmasse,disc_year";

#[component]
pub fn SmallestExoplanetsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || (),
        move |_| async move {
            get_distinct_exoplanets_page(
                1,
                10,
                Some("pl_rade".to_string()),
                Some("asc".to_string()),
                Some(COLUMNS.to_string()),
                None,
            )
            .await
        },
    );

    view! {
        <InsightListPageShell
            eyebrow="Planetary extremes"
            title="Smallest exoplanets by radius"
            intro="A compact ranked view of the smallest confirmed exoplanets using the current archive-backed radius field."
            description="The smallest confirmed exoplanets in the catalog ordered by radius."
            resource=rows_resource
            empty_label="No planet rows available."
        />
    }
}
