use leptos::prelude::*;

use super::format::{alias_label, format_numeric_primary, format_stable_value};
use super::star_color::star_visual_tokens;
use super::star_visual::HostStarVisual;
use crate::server::functions::StellarHostDetail;

#[component]
pub fn HostHeroSection(host: StellarHostDetail) -> impl IntoView {
    let spectype = host
        .star
        .spectype
        .as_ref()
        .map(|value| value.value.clone())
        .unwrap_or_else(|| "Unclassified star".to_string());
    let distance = host.system.distance.as_ref().map(format_numeric_primary);
    let planet_count = host.system.planet_count.as_ref().map(format_stable_value);
    let star_tokens =
        star_visual_tokens(host.star.teff.as_ref().map(|teff| teff.value));
    let aliases = host
        .identity
        .aliases
        .iter()
        .filter(|(_, values)| !values.is_empty())
        .map(|(label, values)| {
            format!("{}: {}", alias_label(label), values.join(", "))
        })
        .collect::<Vec<_>>();

    view! {
        <section class="relative overflow-hidden rounded-[2rem] border border-amber-200/10 bg-[radial-gradient(circle_at_top,_rgba(251,191,36,0.18),_transparent_35%),linear-gradient(135deg,_rgba(15,23,42,0.96),_rgba(30,41,59,0.92))] px-6 py-8 shadow-2xl shadow-amber-950/20 md:px-10 md:py-12">
            <div class="relative grid gap-8 lg:grid-cols-[1.3fr_0.9fr] lg:items-center">
                <div class="space-y-5">
                    <div class="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1 text-xs uppercase tracking-[0.22em] text-slate-300">
                        <span>"Stellar Host"</span>
                        <span>"Profile"</span>
                    </div>
                    <div class="space-y-3">
                        <h1 class="text-4xl font-semibold tracking-tight text-white md:text-6xl">
                            {host.hostname.clone()}
                        </h1>
                        <p class="max-w-2xl text-base text-slate-300 md:text-lg">
                            {spectype}
                            {distance.map(|value| format!(" • {}", value)).unwrap_or_default()}
                        </p>
                    </div>

                    <div class="grid gap-3 sm:grid-cols-3">
                        <HeroStat
                            label="Records"
                            value=host.provenance.record_count.to_string()
                            hint="literature rows"
                        />
                        <HeroStat
                            label="Planets"
                            value=planet_count.unwrap_or_else(|| "—".to_string())
                            hint="system count"
                        />
                        <HeroStat
                            label="References"
                            value=(host.provenance.stellar_refs.len() + host.provenance.system_refs.len()).to_string()
                            hint="distinct source labels"
                        />
                    </div>

                    {if aliases.is_empty() {
                        view! { <div></div> }.into_any()
                    } else {
                        view! {
                            <div class="flex flex-wrap gap-2">
                                {aliases.into_iter().map(|alias| view! {
                                    <span class="rounded-full border border-slate-700 bg-slate-950/40 px-3 py-1 text-sm text-slate-300">
                                        {alias}
                                    </span>
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }}
                </div>

                <div class="flex items-center justify-center">
                    <HostStarVisual tokens=star_tokens />
                </div>
            </div>
        </section>
    }
}

#[component]
fn HeroStat(
    label: &'static str,
    value: String,
    hint: &'static str,
) -> impl IntoView {
    view! {
        <div class="rounded-2xl border border-white/10 bg-white/5 px-4 py-4 backdrop-blur">
            <p class="text-xs uppercase tracking-[0.18em] text-slate-400">{label}</p>
            <p class="mt-2 text-2xl font-semibold text-white">{value}</p>
            <p class="mt-1 text-sm text-slate-400">{hint}</p>
        </div>
    }
}
