use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use leptos_router::lazy_route;

use super::comparison::ScaleComparisonSection;
use super::hero::PlanetHeroSection;
use super::records::PlanetRecordsSection;
use super::summary::PlanetSummarySection;
use crate::error_template::{AppError, app_error_view};
use crate::metadata_helpers::{
    canonical_url, decode_path_segment, encode_path_segment,
    exoplanet_detail_description, exoplanet_detail_title,
};
use crate::server::functions::get_exoplanet_detail;
use crate::structured_data::{StructuredData, exoplanet_dataset_schema};

#[derive(Clone)]
pub struct ExoplanetDetailLazy;

#[lazy_route]
impl LazyRoute for ExoplanetDetailLazy {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <ExoplanetDetailPage/> }.into_any()
    }
}

#[component]
pub fn ExoplanetDetailPage() -> impl IntoView {
    let params = use_params_map();
    let pl_name = Memo::new(move |_| {
        let raw = params.read().get("pl_name").unwrap_or_default();
        decode_path_segment(&raw)
    });

    let detail_resource = Resource::new(
        move || pl_name.get(),
        move |name| async move { get_exoplanet_detail(name).await },
    );

    view! {
                <Suspense fallback=move || {
                    view! {
                        <div class="exoplanet-detail-page">
                        <div class="exoplanet-detail-page__container">
                        <div class="exoplanet-detail-page__loading">
                            <div class="exoplanet-detail-page__loading-spinner"></div>
                            <p class="exoplanet-detail-page__loading-label">"Loading exoplanet profile"</p>
                        </div>
                        </div>
                        </div>
                    }
                }>
                    {move || {
                        detail_resource.get().map(|result| match result {
                            Ok(Some(detail)) => view! {
                                <Title text=exoplanet_detail_title(&detail)/>
                                <Meta name="description" content=exoplanet_detail_description(&detail)/>
                                <Link rel="canonical" href=canonical_url(&format!("/exoplanets/{}", encode_path_segment(&detail.pl_name)))/>
                                <StructuredData value=exoplanet_dataset_schema(&detail)/>
                                <div class="exoplanet-detail-page">
                                <div class="exoplanet-detail-page__container">
                                    <A href="/exoplanets" attr:class="exoplanet-detail-page__back-link">
                                        <span>"←"</span>
                                        <span>"Back to Exoplanets"</span>
                                    </A>

                                <div class="exoplanet-detail-page__content">
                                    <PlanetHeroSection detail=detail.clone() />
                                    <PlanetSummarySection detail=detail.clone() />
                                    <ScaleComparisonSection detail=detail.clone() />
                                    <PlanetRecordsSection detail=detail />
                                </div>
                                </div>
                                </div>
                            }
                            .into_any(),
                            Ok(None) => app_error_view(AppError::NotFound),
                            Err(_) => app_error_view(AppError::InternalServerError),
                        })
                    }}
                </Suspense>
    }
}
