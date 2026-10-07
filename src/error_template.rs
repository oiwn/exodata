use crate::i18n::use_i18n;
use crate::locale::localized_path;
use crate::metadata_helpers::title_with_site;
use leptos::error::Errors;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;
use std::fmt;

#[derive(Clone, Debug)]
pub enum AppError {
    NotFound,
    InternalServerError,
}

impl AppError {
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::NotFound => 404,
            AppError::InternalServerError => 500,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound => write!(f, "Not Found"),
            AppError::InternalServerError => write!(f, "Unable to Load Profile"),
        }
    }
}

impl std::error::Error for AppError {}

pub fn app_error_view(error: AppError) -> AnyView {
    let mut errors = Errors::default();
    errors.insert_with_default_key(error);
    view! { <ErrorTemplate outside_errors=errors/> }.into_any()
}

#[component]
pub fn ErrorTemplate(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional)] errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => RwSignal::new(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors found and we expected errors!"),
        },
    };
    let errors = errors.get_untracked();

    let errors: Vec<AppError> = errors
        .into_iter()
        .filter_map(|(_k, v)| v.downcast_ref::<AppError>().cloned())
        .collect();

    let primary_error = errors
        .first()
        .cloned()
        .unwrap_or(AppError::InternalServerError);
    let locale = use_i18n().get_locale_untracked();
    let overview_href = localized_path("/", locale);
    let hosts_href = localized_path("/stellarhosts", locale);
    let planets_href = localized_path("/exoplanets", locale);

    #[cfg(feature = "ssr")]
    {
        use axum::http::StatusCode;
        use leptos_axum::ResponseOptions;
        let response = use_context::<ResponseOptions>();
        if let Some(response) = response {
            let status = StatusCode::from_u16(primary_error.status_code())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            response.set_status(status);
        }
    }

    let error_code = primary_error.status_code().to_string();
    let error_code_badge = error_code.clone();
    let error_title = primary_error.to_string();
    let eyebrow = if matches!(primary_error, AppError::NotFound) {
        "Uncharted Route"
    } else {
        "Application Error"
    };
    let description = if matches!(primary_error, AppError::NotFound) {
        "The page you requested is not in this catalog. The route may be incorrect, or the object may not have a published page yet."
    } else {
        "We could not load this profile right now. Please try again, or return to the catalog."
    };
    let region_detail = if matches!(primary_error, AppError::NotFound) {
        "No published page at this path"
    } else {
        "Profile temporarily unavailable"
    };

    view! {
        <Title text=title_with_site(&error_title)/>
        <Meta name="robots" content="noindex"/>
        <div class="min-h-[calc(100vh-8rem)] bg-[radial-gradient(circle_at_top,_rgba(129,140,248,0.18),_transparent_35%),linear-gradient(180deg,_#040816_0%,_#070b1d_100%)]">
            <div class="mx-auto flex max-w-7xl flex-col gap-8 px-6 py-10 lg:px-8 lg:py-14">
                <A
                    href=overview_href.clone()
                    attr:class="inline-flex items-center gap-2 text-sm font-medium text-slate-400 transition-colors hover:text-white"
                >
                    <span>"←"</span>
                    <span>"Back to Overview"</span>
                </A>

                <section class="relative overflow-hidden rounded-[2rem] border border-white/10 bg-slate-900/70 px-8 py-10 shadow-[0_30px_80px_rgba(2,6,23,0.55)] backdrop-blur-sm lg:px-10 lg:py-12">
                    <div class="absolute inset-0 bg-[radial-gradient(circle_at_20%_20%,_rgba(96,165,250,0.18),_transparent_24%),radial-gradient(circle_at_78%_38%,_rgba(250,204,21,0.24),_transparent_18%),radial-gradient(circle_at_70%_70%,_rgba(148,163,184,0.12),_transparent_28%)]"></div>
                    <div class="absolute right-[-5rem] top-[-4rem] h-72 w-72 rounded-full bg-amber-300/20 blur-3xl"></div>
                    <div class="absolute left-[-4rem] bottom-[-5rem] h-64 w-64 rounded-full bg-blue-500/15 blur-3xl"></div>

                    <div class="relative grid gap-10 lg:grid-cols-[minmax(0,1.1fr)_24rem] lg:items-center">
                        <div class="space-y-6">
                            <div class="inline-flex items-center gap-3 rounded-full border border-slate-700/80 bg-slate-800/70 px-4 py-2 text-xs font-semibold uppercase tracking-[0.35em] text-slate-300">
                                <span class="h-3 w-3 rounded-full bg-slate-100 shadow-[0_0_14px_rgba(255,255,255,0.6)]"></span>
                                <span>{eyebrow}</span>
                            </div>

                            <div class="space-y-3">
                                <div class="text-sm font-semibold uppercase tracking-[0.4em] text-amber-300/90">
                                    {error_code_badge}
                                </div>
                                <h1 class="max-w-3xl text-5xl font-bold tracking-tight text-white md:text-6xl">
                                    {error_title}
                                </h1>
                                <p class="max-w-2xl text-lg leading-8 text-slate-300">
                                    {description}
                                </p>
                            </div>

                            <div class="grid gap-4 sm:grid-cols-3">
                                <InfoCard
                                    label="Status"
                                    value=error_code.clone()
                                    detail="HTTP response code"
                                />
                                <InfoCard
                                    label="Suggested"
                                    value="Overview".to_string()
                                    detail="Start from the main catalog"
                                />
                                <InfoCard
                                    label="Browse"
                                    value="Tables".to_string()
                                    detail="Jump to stellar hosts or exoplanets"
                                />
                            </div>

                            <div class="flex flex-wrap gap-3 pt-2">
                                <A
                                    href=overview_href
                                    attr:class="inline-flex items-center gap-2 rounded-full bg-white px-5 py-3 text-sm font-semibold text-slate-950 transition-transform hover:scale-[1.02]"
                                >
                                    <span>"Open Overview"</span>
                                </A>
                                <A
                                    href=hosts_href
                                    attr:class="inline-flex items-center gap-2 rounded-full border border-slate-600 bg-slate-800/70 px-5 py-3 text-sm font-semibold text-slate-100 transition-colors hover:border-slate-400 hover:bg-slate-800"
                                >
                                    <span>"Browse Stellar Hosts"</span>
                                </A>
                                <A
                                    href=planets_href
                                    attr:class="inline-flex items-center gap-2 rounded-full border border-slate-600 bg-slate-800/70 px-5 py-3 text-sm font-semibold text-slate-100 transition-colors hover:border-slate-400 hover:bg-slate-800"
                                >
                                    <span>"Browse Exoplanets"</span>
                                </A>
                            </div>
                        </div>

                        <div class="relative flex items-center justify-center">
                            <div class="absolute h-72 w-72 rounded-full bg-amber-200/20 blur-3xl"></div>
                            <div class="relative flex h-80 w-80 items-center justify-center">
                                <div class="absolute h-44 w-44 rounded-full bg-[radial-gradient(circle_at_32%_28%,_rgba(255,255,255,0.95),_rgba(255,255,255,0.18)_20%,_rgba(250,204,21,0.92)_42%,_rgba(180,83,9,0.9)_100%)] shadow-[0_0_80px_rgba(250,204,21,0.28)]"></div>
                                <div class="absolute h-64 w-64 rounded-full border border-white/10"></div>
                                <div class="absolute h-64 w-64 rotate-[24deg] rounded-full border border-slate-500/35"></div>
                                <div class="absolute right-14 top-14 h-4 w-4 rounded-full bg-blue-300 shadow-[0_0_18px_rgba(125,211,252,0.9)]"></div>
                                <div class="absolute bottom-16 left-12 h-2.5 w-2.5 rounded-full bg-slate-200 shadow-[0_0_12px_rgba(255,255,255,0.8)]"></div>
                                <div class="absolute bottom-10 rounded-full border border-slate-700/80 bg-slate-950/90 px-5 py-3 text-center shadow-xl">
                                    <div class="text-[0.7rem] font-semibold uppercase tracking-[0.3em] text-slate-400">
                                        "Approximate Region"
                                    </div>
                                    <div class="text-sm font-semibold text-white">
                                        {region_detail}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>
            </div>
        </div>
    }
}

#[component]
fn InfoCard(
    label: &'static str,
    value: String,
    detail: &'static str,
) -> impl IntoView {
    view! {
        <div class="rounded-3xl border border-white/8 bg-slate-800/55 px-5 py-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]">
            <div class="text-xs font-semibold uppercase tracking-[0.28em] text-slate-400">
                {label}
            </div>
            <div class="mt-2 text-4xl font-semibold leading-none text-white">
                {value}
            </div>
            <div class="mt-3 text-sm text-slate-400">
                {detail}
            </div>
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::i18n::{I18nContextProvider, Locale};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use leptos_router::components::Router;

    async fn render(error: AppError, locale: Locale) -> (String, StatusCode) {
        tokio::task::LocalSet::new()
            .run_until(async move {
                // Route registration initializes the SSR executor before rendering.
                let _ = leptos_axum::generate_route_list(|| view! { <div/> });
                let handler = leptos_axum::render_app_async_with_context(
                    || {},
                    move || {
                        let error = error.clone();
                        view! {
                        <html>
                            <head></head>
                            <body>
                        <I18nContextProvider enable_cookie=false>
                            <Router>
                                {move || {
                                    use_i18n().set_locale(locale);
                                    app_error_view(error.clone())
                                }}
                            </Router>
                        </I18nContextProvider>
                            </body>
                        </html>
                        }
                    },
                );
                let response = handler(
                    Request::builder().uri("/").body(Body::empty()).unwrap(),
                )
                .await;
                let status = response.status();
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap();
                (String::from_utf8(body.to_vec()).unwrap(), status)
            })
            .await
    }

    #[tokio::test]
    async fn detail_error_template_returns_404_and_locale_recovery_links() {
        let (html, status) = render(AppError::NotFound, Locale::ja).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(html.contains("Not Found"));
        assert!(html.contains("href=\"/ja/stellarhosts\""));
        assert!(html.contains("href=\"/ja/exoplanets\""));
        assert!(!html.contains("error running server function"));
    }

    #[tokio::test]
    async fn detail_error_template_returns_500_without_not_found_copy() {
        let (html, status) =
            render(AppError::InternalServerError, Locale::en).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(html.contains("Unable to Load Profile"));
        assert!(html.contains("Please try again"));
        assert!(!html.contains("No published page at this path"));
    }
}
