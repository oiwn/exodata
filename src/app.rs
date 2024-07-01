use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn TableWrapper() -> impl IntoView {
    provide_meta_context();

    view! {
        <main>
            <h1>Table here!</h1>
        </main>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {


        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/exoplanets-catalog.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="table" view=TableWrapper/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    static BTN_CLASS: &str = "flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 \
        text-sm font-semibold leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline \
        focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600";

    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Website!"</h1>
        <button on:click=on_click class=BTN_CLASS>"Click Me: " {count}</button>
    }
}
