use crate::components::{
    about::AboutLazy, exoplanet_detail::ExoplanetDetailLazy,
    exoplanets_table::ExoplanetsTableLazy, footer::VersionFooter,
    google_analytics::GoogleAnalytics, navbar::Navbar, overview::OverviewLazy,
    stellarhost_detail::StellarHostDetailLazy,
    stellarhosts_table::StellarHostsTableLazy,
};
use crate::error_template::{AppError, ErrorTemplate};
use crate::metadata::{METADATA_SCRIPT_ID, provide_app_metadata_store};

use leptos::error::Errors;
use leptos::hydration::HydrationScripts;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::Lazy;
use leptos_router::SsrMode;
use leptos_router::StaticSegment;
use leptos_router::components::*;
use leptos_router::path;

pub fn shell(
    options: LeptosOptions,
    ga_measurement_id: Option<String>,
    metadata_json: String,
) -> impl IntoView {
    let metadata_json = metadata_json.replace("</", "<\\/");

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <script id=METADATA_SCRIPT_ID type="application/json">
                    {metadata_json}
                </script>
                <script>"document.documentElement.classList.add('pre-hydration')"</script>
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
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_app_metadata_store();

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
                <Route path=StaticSegment("") view={Lazy::<OverviewLazy>::new()}/>
                <Route
                    path=StaticSegment("stellarhosts")
                    view={Lazy::<StellarHostsTableLazy>::new()}
                    ssr=SsrMode::OutOfOrder
                />
                <Route path=path!("/stellarhosts/:hostname") view={Lazy::<StellarHostDetailLazy>::new()}/>
                <Route
                    path=StaticSegment("exoplanets")
                    view={Lazy::<ExoplanetsTableLazy>::new()}
                    ssr=SsrMode::OutOfOrder
                />
                <Route path=path!("/exoplanets/:pl_name") view={Lazy::<ExoplanetDetailLazy>::new()}/>
                <Route path=StaticSegment("about") view={Lazy::<AboutLazy>::new()}/>
            </Routes>

            <VersionFooter/>
        </Router>
    }
}
