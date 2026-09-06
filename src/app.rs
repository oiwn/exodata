use crate::components::{
    about::AboutLazy,
    docs::DocsLazy,
    exoplanet_detail::ExoplanetDetailLazy,
    exoplanets_table::ExoplanetsTableLazy,
    footer::VersionFooter,
    google_analytics::GoogleAnalytics,
    insights::{InsightDetailLazy, InsightsLazy},
    navbar::Navbar,
    overview::OverviewLazy,
    stellarhost_detail::StellarHostDetailLazy,
    stellarhosts_table::StellarHostsTableLazy,
};
use crate::error_template::{AppError, ErrorTemplate};
use crate::metadata::{METADATA_SCRIPT_ID, provide_app_metadata_store};
use crate::metadata_helpers::SITE_NAME;
use crate::{i18n::*, locale::locale_from_path};

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
        <Stylesheet id="leptos" href="/pkg/exodata.css"/>

        <Title text=SITE_NAME/>

        <I18nContextProvider enable_cookie=false set_lang_attr_on_html=true>
            // content for this welcome page
            <Router>
                <LocaleSync/>
                // Navigation bar (must be inside Router to use location)
                <Navbar/>

                <main role="main">
                    <Routes fallback=|| {
                    let mut outside_errors = Errors::default();
                    outside_errors.insert_with_default_key(AppError::NotFound);
                    view! {
                        <ErrorTemplate outside_errors/>
                    }
                    .into_view()
                }>
                <Route path=StaticSegment("") view={Lazy::<OverviewLazy>::new()} ssr=SsrMode::Async/>
                <Route
                    path=StaticSegment("stellarhosts")
                    view={Lazy::<StellarHostsTableLazy>::new()}
                    ssr=SsrMode::Async
                />
                <Route path=path!("/stellarhosts/:hostname") view={Lazy::<StellarHostDetailLazy>::new()} ssr=SsrMode::Async/>
                <Route
                    path=StaticSegment("exoplanets")
                    view={Lazy::<ExoplanetsTableLazy>::new()}
                    ssr=SsrMode::Async
                />
                <Route path=path!("/exoplanets/:pl_name") view={Lazy::<ExoplanetDetailLazy>::new()} ssr=SsrMode::Async/>
                <Route path=StaticSegment("insights") view={Lazy::<InsightsLazy>::new()} ssr=SsrMode::Async/>
                <Route path=path!("/insights/:slug") view={Lazy::<InsightDetailLazy>::new()} ssr=SsrMode::Async/>
                <Route path=StaticSegment("docs") view={Lazy::<DocsLazy>::new()} ssr=SsrMode::Async/>
                <Route path=path!("/docs/:slug") view={Lazy::<DocsLazy>::new()} ssr=SsrMode::Async/>
                <Route path=StaticSegment("about") view={Lazy::<AboutLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=StaticSegment("zh-CN") view={Lazy::<OverviewLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/zh-CN/stellarhosts") view={Lazy::<StellarHostsTableLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/zh-CN/stellarhosts/:hostname") view={Lazy::<StellarHostDetailLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/zh-CN/exoplanets") view={Lazy::<ExoplanetsTableLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/zh-CN/exoplanets/:pl_name") view={Lazy::<ExoplanetDetailLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/zh-CN/insights") view={Lazy::<InsightsLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/zh-CN/insights/:slug") view={Lazy::<InsightDetailLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/zh-CN/docs") view={Lazy::<DocsLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/zh-CN/docs/:slug") view={Lazy::<DocsLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/zh-CN/about") view={Lazy::<AboutLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=StaticSegment("ja") view={Lazy::<OverviewLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/ja/stellarhosts") view={Lazy::<StellarHostsTableLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/ja/stellarhosts/:hostname") view={Lazy::<StellarHostDetailLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/ja/exoplanets") view={Lazy::<ExoplanetsTableLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/ja/exoplanets/:pl_name") view={Lazy::<ExoplanetDetailLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/ja/insights") view={Lazy::<InsightsLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/ja/insights/:slug") view={Lazy::<InsightDetailLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/ja/docs") view={Lazy::<DocsLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/ja/docs/:slug") view={Lazy::<DocsLazy>::new()} ssr=SsrMode::Async/>
                    <Route path=path!("/ja/about") view={Lazy::<AboutLazy>::new()} ssr=SsrMode::Async/>
                    </Routes>
                </main>

                <VersionFooter/>
            </Router>
        </I18nContextProvider>
    }
}

#[component]
fn LocaleSync() -> impl IntoView {
    use leptos_router::hooks::use_location;

    let location = use_location();
    let i18n = use_i18n();
    i18n.set_locale_untracked(locale_from_path(
        &location.pathname.get_untracked(),
    ));

    Effect::new(move |_| {
        i18n.set_locale(locale_from_path(&location.pathname.get()));
    });
}
