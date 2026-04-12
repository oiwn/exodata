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
        <section class="host-detail-section">
            <div class="host-detail-section__header">
                <div>
                    <p class="host-detail-section__eyebrow host-detail-section__eyebrow--summary">"Canonical Summary"</p>
                    <h2 class="host-detail-section__title">"Adopted host values from all rows"</h2>
                </div>
                <p class="host-detail-section__description host-detail-section__description--summary">
                    "Numeric fields use the median of non-null measurements. Disagreement stays visible through ranges, counts, and provenance."
                </p>
            </div>

            <div class="host-detail-summary-grid">
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
        <article class="host-detail-card">
            <p class="host-detail-card__label">{summary.label.clone()}</p>
            <p class="host-detail-card__value">{primary}</p>
            <p class="host-detail-card__body">{range}</p>
            <p class="host-detail-card__meta">{evidence}</p>
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
        <article class="host-detail-card">
            <p class="host-detail-card__label">{summary.label.clone()}</p>
            <p class="host-detail-card__value">{summary.value.clone()}</p>
            <p class="host-detail-card__body">
                {if summary.disputed { "Multiple classifications reported" } else { "Consistent across records" }}
            </p>
            <p class="host-detail-card__meta">{counts}</p>
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
        <article class="host-detail-card">
            <p class="host-detail-card__label">{summary.label.clone()}</p>
            <p class="host-detail-card__value">{value}</p>
            <p class="host-detail-card__body">
                {if summary.disputed { "System counts disagree across records" } else { "Stable across source rows" }}
            </p>
            <p class="host-detail-card__meta">{distinct}</p>
        </article>
    }
}
