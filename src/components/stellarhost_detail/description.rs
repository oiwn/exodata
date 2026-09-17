use leptos::prelude::*;

/// Generated prose description for a stellar host, rendered after the hero
/// when a description exists. Absent descriptions render nothing.
#[component]
pub fn DescriptionSection(description: Option<String>) -> impl IntoView {
    move || {
        description.as_ref().map(|html| {
            view! {
                <section class="host-description">
                    <div
                        class="host-description__prose"
                        inner_html=html.clone()
                    ></div>
                </section>
            }
        })
    }
}
