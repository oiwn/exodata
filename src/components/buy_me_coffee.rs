use leptos::prelude::*;

#[component]
pub fn BuyMeACoffee(
    /// Buy Me a Coffee username/slug. When None/empty, renders nothing.
    #[prop(optional)]
    slug: Option<String>,
) -> impl IntoView {
    let slug = slug.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    if let Some(slug) = slug {
        let img_src = format!(
            "https://img.buymeacoffee.com/button-api/?text=Buy me a coffee&emoji=&slug={}&button_colour=BD5FFF&font_colour=ffffff&font_family=Lato&outline_colour=000000&coffee_colour=FFDD00",
            slug
        );
        let link_href = format!("https://www.buymeacoffee.com/{}", slug);

        view! {
            <a href=link_href target="_blank" rel="noopener noreferrer">
                <img src=img_src alt="Buy Me A Coffee" />
            </a>
        }
        .into_any()
    } else {
        view! { <></> }.into_any()
    }
}
