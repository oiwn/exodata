use leptos::prelude::*;
use leptos_router::LazyRoute;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use leptos_router::lazy_route;

use super::hero::HostHeroSection;
use super::planets::PlanetsSection;
use super::provenance::ProvenanceSection;
use super::summary::CanonicalSummarySection;
use crate::server::functions::{get_planets_for_host, get_stellar_host_detail};

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
        params
            .read()
            .get("hostname")
            .unwrap_or_default()
            .replace("%20", " ")
            .replace("%23", "#")
    });

    let host_resource = Resource::new(
        move || hostname.get(),
        move |name| async move { get_stellar_host_detail(name).await },
    );

    let planets_resource = Resource::new(
        move || hostname.get(),
        move |name| async move { get_planets_for_host(name).await },
    );

    view! {
        <div class="min-h-screen bg-[linear-gradient(180deg,_#020617_0%,_#0f172a_45%,_#111827_100%)]">
            <div class="mx-auto max-w-7xl px-4 py-8 md:px-6 md:py-10">
                <A
                    href="/stellarhosts"
                    attr:class="inline-flex items-center gap-2 text-sm text-slate-400 transition-colors hover:text-white"
                >
                    <span>"←"</span>
                    <span>"Back to Stellar Hosts"</span>
                </A>

                <Suspense fallback=move || {
                    view! {
                        <div class="flex min-h-[60vh] flex-col items-center justify-center gap-4">
                            <div class="h-16 w-16 animate-spin rounded-full border-4 border-amber-400/20 border-t-amber-300"></div>
                            <p class="text-sm uppercase tracking-[0.2em] text-slate-400">"Loading stellar profile"</p>
                        </div>
                    }
                }>
                    {move || {
                        let host_data = host_resource.get();
                        let planets_data = planets_resource.get();

                        match (host_data, planets_data) {
                            (Some(Ok(host)), Some(Ok(planets))) => view! {
                                <div class="mt-6 space-y-10 pb-14">
                                    <HostHeroSection host=host.clone() />
                                    <CanonicalSummarySection host=host.clone() />
                                    <PlanetsSection planets=planets />
                                    <ProvenanceSection host=host />
                                </div>
                            }
                            .into_any(),
                            (Some(Err(error)), _) | (_, Some(Err(error))) => view! {
                                <div class="mt-10 rounded-[1.5rem] border border-rose-500/40 bg-rose-950/30 px-6 py-5 text-rose-100">
                                    <h2 class="text-xl font-semibold">"Error Loading Host"</h2>
                                    <p class="mt-2 text-sm text-rose-200">{error.to_string()}</p>
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
