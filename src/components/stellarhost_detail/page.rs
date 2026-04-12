use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::LazyRoute;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use leptos_router::lazy_route;

use super::hero::HostHeroSection;
use super::planets::PlanetsSection;
use super::provenance::ProvenanceSection;
use super::summary::CanonicalSummarySection;
use crate::metadata_helpers::{
    canonical_url, decode_path_segment, encode_path_segment,
    stellarhost_detail_description, stellarhost_detail_title, title_with_site,
};
use crate::server::functions::{get_planets_for_host, get_stellar_host_detail};
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

    let fallback_title =
        move || title_with_site(&format!("{} Stellar Host", hostname.get()));
    let fallback_description = move || {
        format!(
            "Explore the stellar host profile, system summary, and planet list for {}.",
            hostname.get()
        )
    };
    let canonical_href = canonical_url(&format!(
        "/stellarhosts/{}",
        encode_path_segment(&hostname.get_untracked())
    ));

    let host_resource = Resource::new(
        move || hostname.get(),
        move |name| async move { get_stellar_host_detail(name).await },
    );

    let planets_resource = Resource::new(
        move || hostname.get(),
        move |name| async move { get_planets_for_host(name).await },
    );

    view! {
        <Title text=move || fallback_title()/>
        <Meta name="description" content=move || fallback_description()/>
        <Link rel="canonical" href=canonical_href.clone()/>
        <div class="stellarhost-detail-page">
            <div class="stellarhost-detail-page__container">
                <A
                    href="/stellarhosts"
                    attr:class="stellarhost-detail-page__back-link"
                >
                    <span>"←"</span>
                    <span>"Back to Stellar Hosts"</span>
                </A>

                <Suspense fallback=move || {
                    view! {
                        <div class="stellarhost-detail-page__loading">
                            <div class="stellarhost-detail-page__loading-spinner"></div>
                            <p class="stellarhost-detail-page__loading-label">"Loading stellar profile"</p>
                        </div>
                    }
                }>
                    {move || {
                        let host_data = host_resource.get();
                        let planets_data = planets_resource.get();

                        match (host_data, planets_data) {
                            (Some(Ok(host)), Some(Ok(planets))) => view! {
                                <Title text=stellarhost_detail_title(&host)/>
                                <Meta name="description" content=stellarhost_detail_description(&host)/>
                                <StructuredData value=stellarhost_dataset_schema(&host)/>
                                <div class="stellarhost-detail-page__content">
                                    <HostHeroSection host=host.clone() />
                                    <CanonicalSummarySection host=host.clone() />
                                    <PlanetsSection planets=planets />
                                    <ProvenanceSection host=host />
                                </div>
                            }
                            .into_any(),
                            (Some(Err(error)), _) | (_, Some(Err(error))) => view! {
                                <div class="stellarhost-detail-page__error">
                                    <h2 class="stellarhost-detail-page__error-title">"Error Loading Host"</h2>
                                    <p class="stellarhost-detail-page__error-message">{error.to_string()}</p>
                                </div>
                            }
                            .into_any(),
                            _ => view! { <div></div> }.into_any(),
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
