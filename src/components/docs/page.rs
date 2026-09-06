use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::hooks::use_params_map;
use leptos_router::lazy_route;

use crate::metadata_helpers::{canonical_url, title_with_site};

use super::{registry, render};

#[derive(Clone)]
pub struct DocsLazy;

#[lazy_route]
impl LazyRoute for DocsLazy {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <DocsPage/> }.into_any()
    }
}

#[component]
pub fn DocsPage() -> impl IntoView {
    let params = use_params_map();
    let slug = Memo::new(move |_| params.read().get("slug").unwrap_or_default());

    view! {
        {move || {
            let current_slug = slug.get();
            match registry::find_page(&current_slug) {
                Some(page) => view! { <DocContent page/> }.into_any(),
                None => view! { <DocNotFound slug=current_slug/> }.into_any(),
            }
        }}
    }
}

#[component]
fn DocContent(page: &'static registry::DocPage) -> impl IntoView {
    let html = render::render_markdown(page.markdown);
    let path = registry::path_for(page);
    let keywords = page.keywords.join(", ");

    view! {
        <Title text=title_with_site(page.title)/>
        <Meta name="description" content=page.description/>
        <Meta name="keywords" content=keywords/>
        <Link rel="canonical" href=canonical_url(&path)/>

        <div class="docs-page">
            <div class="docs-page__container">
                <article class="docs-content" inner_html=html></article>
            </div>
        </div>
    }
}

#[component]
fn DocNotFound(slug: String) -> impl IntoView {
    let path = if slug.is_empty() {
        "/docs".to_string()
    } else {
        format!("/docs/{slug}")
    };

    view! {
        <Title text=title_with_site("Docs Not Found")/>
        <Meta name="description" content="Documentation page not found."/>
        <Link rel="canonical" href=canonical_url(&path)/>

        <div class="docs-page">
            <div class="docs-page__container">
                <section class="docs-content">
                    <h1>"Documentation not found"</h1>
                    <p>
                        <a href="/docs">"Open the documentation index."</a>
                    </p>
                </section>
            </div>
        </div>
    }
}
