use leptos::prelude::*;

use super::format::{format_json_value, format_number, format_numeric_primary};
use crate::server::functions::{
    CategoricalFieldSummary, NumericFieldSummary, StableValueSummary,
    StellarHostDetail,
};

#[component]
pub fn CanonicalSummarySection(host: StellarHostDetail) -> impl IntoView {
    let primary_cards = vec![
        host.star.teff.clone(),
        host.star.mass.clone(),
        host.star.radius.clone(),
        host.star.age.clone(),
        host.star.luminosity.clone(),
        host.system.distance.clone(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let spectype = host.star.spectype.clone();
    let planet_count = host.system.planet_count.clone();

    view! {
        <section class="space-y-6">
            <div class="flex items-end justify-between gap-4">
                <div>
                    <p class="text-sm uppercase tracking-[0.18em] text-amber-300">"Canonical Summary"</p>
                    <h2 class="mt-2 text-3xl font-semibold text-white">"Adopted host values from all rows"</h2>
                </div>
                <p class="max-w-xl text-sm text-slate-400">
                    "Numeric fields use the median of non-null measurements. Disagreement stays visible through ranges, counts, and provenance."
                </p>
            </div>

            <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                {primary_cards.into_iter().map(|summary| view! {
                    <NumericSummaryCard summary=summary />
                }).collect::<Vec<_>>()}
                {spectype.map(|summary| view! { <CategoricalSummaryCard summary=summary /> })}
                {planet_count.map(|summary| view! { <StableSummaryCard summary=summary /> })}
            </div>
        </section>
    }
}

#[component]
fn NumericSummaryCard(summary: NumericFieldSummary) -> impl IntoView {
    let primary = format_numeric_primary(&summary);
    let range = if summary.disputed {
        format!(
            "{} to {}",
            format_number(summary.min),
            format_number(summary.max)
        )
    } else {
        "single adopted value".to_string()
    };
    let evidence = format!(
        "{} values • {} distinct",
        summary.measurement_count, summary.distinct_count
    );

    view! {
        <article class="rounded-[1.5rem] border border-slate-800 bg-slate-950/70 p-5 shadow-lg shadow-slate-950/30">
            <p class="text-xs uppercase tracking-[0.18em] text-slate-400">{summary.label.clone()}</p>
            <p class="mt-3 text-3xl font-semibold text-white">{primary}</p>
            <p class="mt-3 text-sm text-slate-300">{range}</p>
            <p class="mt-1 text-sm text-slate-500">{evidence}</p>
        </article>
    }
}

#[component]
fn CategoricalSummaryCard(summary: CategoricalFieldSummary) -> impl IntoView {
    let counts = summary
        .counts
        .iter()
        .map(|item| format!("{} ({})", item.value, item.count))
        .collect::<Vec<_>>()
        .join(", ");

    view! {
        <article class="rounded-[1.5rem] border border-slate-800 bg-slate-950/70 p-5 shadow-lg shadow-slate-950/30">
            <p class="text-xs uppercase tracking-[0.18em] text-slate-400">{summary.label.clone()}</p>
            <p class="mt-3 text-3xl font-semibold text-white">{summary.value.clone()}</p>
            <p class="mt-3 text-sm text-slate-300">
                {if summary.disputed { "Multiple classifications reported" } else { "Consistent across records" }}
            </p>
            <p class="mt-1 text-sm text-slate-500">{counts}</p>
        </article>
    }
}

#[component]
fn StableSummaryCard(summary: StableValueSummary) -> impl IntoView {
    let value =
        format_json_value(&summary.value, summary.unit.as_deref().unwrap_or(""));
    let distinct = summary
        .distinct_values
        .iter()
        .map(|value| {
            format_json_value(value, summary.unit.as_deref().unwrap_or(""))
        })
        .collect::<Vec<_>>()
        .join(", ");

    view! {
        <article class="rounded-[1.5rem] border border-slate-800 bg-slate-950/70 p-5 shadow-lg shadow-slate-950/30">
            <p class="text-xs uppercase tracking-[0.18em] text-slate-400">{summary.label.clone()}</p>
            <p class="mt-3 text-3xl font-semibold text-white">{value}</p>
            <p class="mt-3 text-sm text-slate-300">
                {if summary.disputed { "System counts disagree across records" } else { "Stable across source rows" }}
            </p>
            <p class="mt-1 text-sm text-slate-500">{distinct}</p>
        </article>
    }
}
