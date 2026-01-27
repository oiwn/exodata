use crate::components::about::AboutPage;
use crate::components::exoplanets_table::ExoplanetsTablePage;
use crate::components::google_analytics::GoogleAnalytics;
use crate::components::navbar::Navbar;
use crate::components::overview::OverviewPage;
use crate::components::stellarhost_detail::StellarHostDetailPage;
use crate::components::stellarhosts_table::StellarHostsTablePage;
use crate::error_template::{AppError, ErrorTemplate};
use leptos::error::Errors;
use leptos::hydration::HydrationScripts;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::path;
use leptos_router::StaticSegment;
use leptos_router::components::*;

pub fn shell(
    options: LeptosOptions,
    ga_measurement_id: Option<String>,
) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() />
                {ga_measurement_id.map(|id| view! { <GoogleAnalytics measurement_id=id /> })}
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn TableWrapper() -> impl IntoView {
    provide_meta_context();

    view! {
        <main>
            <h1>"Table here!"</h1>
            <p>"Data loading functionality implemented in CLI"</p>
        </main>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/exoplanets-catalog.css"/>

        // sets the document title
        <Title text="Exoplanets Catalog"/>

        // content for this welcome page
        <Router>
            // Navigation bar (must be inside Router to use location)
            <Navbar/>

            <Routes fallback=|| {
                let mut outside_errors = Errors::default();
                outside_errors.insert_with_default_key(AppError::NotFound);
                view! {
                    <ErrorTemplate outside_errors/>
                }
                .into_view()
            }>
                <Route path=StaticSegment("") view=OverviewPage/>
                <Route path=StaticSegment("stellarhosts") view=StellarHostsTablePage/>
                <Route path=path!("/stellarhosts/:hostname") view=StellarHostDetailPage/>
                <Route path=StaticSegment("exoplanets") view=ExoplanetsTablePage/>
                <Route path=StaticSegment("about") view=AboutPage/>
                <Route path=StaticSegment("table") view=TableWrapper/>
            </Routes>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = signal(0);
    static BTN_CLASS: &str = "flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 \
        text-sm font-semibold leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline \
        focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600";

    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <main>
            <h1>"Website!"</h1>
            <button on:click=on_click class=BTN_CLASS>"Click Me: " {count}</button>
        </main>
    }
}
