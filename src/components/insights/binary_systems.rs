use leptos::prelude::*;

use super::common::InsightListPageShell;
use crate::server::functions::get_binary_star_systems_insight;

#[component]
pub fn BinaryStarSystemsPage() -> impl IntoView {
    let rows_resource = Resource::new(
        move || (),
        move |_| async move { get_binary_star_systems_insight().await },
    );

    view! {
        <InsightListPageShell
            eyebrow="System architecture"
            title="Binary planetary systems with planets"
            intro="Confirmed planetary systems where the archive-backed system star count marks the system as a binary."
            description="Binary star systems with confirmed planets, ordered by system-level confirmed planet count."
            resource=rows_resource
            empty_label="No binary star systems available."
        />
    }
}
