use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use leptos_router::lazy_route;

use crate::metadata_helpers::{canonical_url, title_with_site};

#[derive(Clone)]
pub struct AboutLazy;

#[lazy_route]
impl LazyRoute for AboutLazy {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <AboutRedirect/> }.into_any()
    }
}

#[component]
pub fn AboutRedirect() -> impl IntoView {
    set_redirect_response();

    let navigate = use_navigate();
    Effect::new(move |_| {
        navigate(
            "/docs",
            NavigateOptions {
                replace: true,
                ..Default::default()
            },
        );
    });

    view! {
        <Title text=title_with_site("Documentation")/>
        <Meta name="description" content="Documentation has moved to /docs."/>
        <Link rel="canonical" href=canonical_url("/docs")/>
        <main class="docs-page">
            <div class="docs-page__container">
                <section class="docs-content">
                    <h1>"Documentation moved"</h1>
                    <p>
                        <a href="/docs">"Open the documentation."</a>
                    </p>
                </section>
            </div>
        </main>
    }
}

#[cfg(feature = "ssr")]
fn set_redirect_response() {
    use axum::http::{HeaderValue, StatusCode, header};
    use leptos_axum::ResponseOptions;

    if let Some(response) = use_context::<ResponseOptions>() {
        response.set_status(StatusCode::MOVED_PERMANENTLY);
        response
            .insert_header(header::LOCATION, HeaderValue::from_static("/docs"));
    }
}

#[cfg(not(feature = "ssr"))]
fn set_redirect_response() {}
