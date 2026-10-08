use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use leptos_router::lazy_route;

use super::comparison::StarScaleComparisonSection;
use super::description::DescriptionSection;
use super::hero::HostHeroSection;
use super::planets::PlanetsSection;
use super::provenance::ProvenanceSection;
use super::summary::CanonicalSummarySection;
use crate::error_template::{AppError, app_error_view};
use crate::metadata_helpers::{
    canonical_url, decode_path_segment, encode_path_segment,
    stellarhost_detail_description, stellarhost_detail_title,
};
use crate::server::functions::{
    get_host_description, get_planets_for_host, get_stellar_host_detail,
};
use crate::structured_data::{StructuredData, stellarhost_dataset_schema};

#[derive(Clone)]
pub struct StellarHostDetailLazy;

#[lazy_route]
impl LazyRoute for StellarHostDetailLazy {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <StellarHostDetailPage/> }.into_any()
    }
}

#[component]
pub fn StellarHostDetailPage() -> impl IntoView {
    let params = use_params_map();
    let hostname = Memo::new(move |_| {
        let raw = params.read().get("hostname").unwrap_or_default();
        decode_path_segment(&raw)
    });

    let host_resource = Resource::new(
        move || hostname.get(),
        move |name| async move { get_stellar_host_detail(name).await },
    );

    let planets_resource = Resource::new(
        move || hostname.get(),
        move |name| async move { get_planets_for_host(name).await },
    );

    let description_resource = Resource::new(
        move || hostname.get(),
        move |name| async move { get_host_description(name).await },
    );

    view! {
                <Suspense fallback=move || {
                    view! {
                        <div class="stellarhost-detail-page">
                        <div class="stellarhost-detail-page__container">
                        <div class="stellarhost-detail-page__loading">
                            <div class="stellarhost-detail-page__loading-spinner"></div>
                            <p class="stellarhost-detail-page__loading-label">"Loading stellar profile"</p>
                        </div>
                        </div>
                        </div>
                    }
                }>
                    {move || {
                        let host_data = host_resource.get();
                        let planets_data = planets_resource.get();
                        let description = description_resource
                            .get()
                            .and_then(|result| result.ok().flatten());

                        match (host_data, planets_data) {
                            (Some(Ok(Some(host))), Some(Ok(planets))) => view! {
                                <Title text=stellarhost_detail_title(&host)/>
                                <Meta name="description" content=stellarhost_detail_description(&host)/>
                                <Link rel="canonical" href=canonical_url(&format!("/stellarhosts/{}", encode_path_segment(&host.hostname)))/>
                                <StructuredData value=stellarhost_dataset_schema(&host)/>
                                <div class="stellarhost-detail-page">
                                <div class="stellarhost-detail-page__container">
                                    <A href="/stellarhosts" attr:class="stellarhost-detail-page__back-link">
                                        <span>"←"</span>
                                        <span>"Back to Stellar Hosts"</span>
                                    </A>
                                <div class="stellarhost-detail-page__content">
                                    <HostHeroSection host=host.clone() />
                                    <CanonicalSummarySection host=host.clone() />
                                    <StarScaleComparisonSection host=host.clone() />
                                    <PlanetsSection planets=planets />
                                    <DescriptionSection description=description />
                                    <ProvenanceSection host=host />
                                </div>
                                </div>
                                </div>
                            }
                            .into_any(),
                            (Some(Ok(None)), _) => app_error_view(AppError::NotFound),
                            (Some(Err(_)), _) | (_, Some(Err(_))) => app_error_view(AppError::InternalServerError),
                            _ => view! { <div></div> }.into_any(),
                        }
                    }}
                </Suspense>
    }
}
