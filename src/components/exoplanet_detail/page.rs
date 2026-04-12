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
use crate::metadata_helpers::{
    canonical_url, decode_path_segment, encode_path_segment,
    exoplanet_detail_description, exoplanet_detail_title, title_with_site,
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

    let fallback_title =
        move || title_with_site(&format!("{} Exoplanet", pl_name.get()));
    let fallback_description = move || {
        format!(
            "Explore measurements and source records for the exoplanet {}.",
            pl_name.get()
        )
    };
    let canonical_href = canonical_url(&format!(
        "/exoplanets/{}",
        encode_path_segment(&pl_name.get_untracked())
    ));

    let detail_resource = Resource::new(
        move || pl_name.get(),
        move |name| async move { get_exoplanet_detail(name).await },
    );

    view! {
        <Title text=move || fallback_title()/>
        <Meta name="description" content=move || fallback_description()/>
        <Link rel="canonical" href=canonical_href.clone()/>

        <div class="exoplanet-detail-page">
            <div class="exoplanet-detail-page__container">
                <A
                    href="/exoplanets"
                    attr:class="exoplanet-detail-page__back-link"
                >
                    <span>"←"</span>
                    <span>"Back to Exoplanets"</span>
                </A>

                <Suspense fallback=move || {
                    view! {
                        <div class="exoplanet-detail-page__loading">
                            <div class="exoplanet-detail-page__loading-spinner"></div>
                            <p class="exoplanet-detail-page__loading-label">"Loading exoplanet profile"</p>
                        </div>
                    }
                }>
                    {move || {
                        detail_resource.get().map(|result| match result {
                            Ok(detail) => view! {
                                <Title text=exoplanet_detail_title(&detail)/>
                                <Meta name="description" content=exoplanet_detail_description(&detail)/>
                                <StructuredData value=exoplanet_dataset_schema(&detail)/>

                                <div class="exoplanet-detail-page__content">
                                    <PlanetHeroSection detail=detail.clone() />
                                    <PlanetSummarySection detail=detail.clone() />
                                    <ScaleComparisonSection detail=detail.clone() />
                                    <PlanetRecordsSection detail=detail />
                                </div>
                            }
                            .into_any(),
                            Err(err) => view! {
                                <div class="exoplanet-detail-page__error">
                                    <h2 class="exoplanet-detail-page__error-title">"Error Loading Planet"</h2>
                                    <p class="exoplanet-detail-page__error-message">{err.to_string()}</p>
                                </div>
                            }
                            .into_any(),
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
