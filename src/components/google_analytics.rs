use leptos::prelude::*;

#[component]
pub fn GoogleAnalytics(
    /// GA4 measurement ID (e.g., "G-XXXXXXX"). When None/empty, renders nothing.
    #[prop(optional)]
    measurement_id: Option<String>,
) -> impl IntoView {
    let measurement_id = measurement_id.and_then(|id| {
        let trimmed = id.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    if let Some(measurement_id) = measurement_id {
        let measurement_id_ref = measurement_id.as_str();
        let script_src = format!(
            "https://www.googletagmanager.com/gtag/js?id={}",
            measurement_id_ref
        );
        let inline = format!(
            "window.dataLayer = window.dataLayer || [];\n\
function gtag(){{dataLayer.push(arguments);}}\n\
gtag('js', new Date());\n\
gtag('config', '{}');",
            measurement_id_ref
        );

        view! {
            <>
                <script async src=script_src></script>
                <script>{inline}</script>
            </>
        }
        .into_any()
    } else {
        view! { <></> }.into_any()
    }
}
