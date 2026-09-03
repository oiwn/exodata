use leptos::prelude::*;
use leptos::serde_json::Value;
use leptos_router::components::A;

use super::format::{format_number, format_value};
use crate::metadata_helpers::encode_path_segment;
use crate::server::functions::{
    CategoricalFieldSummary, ExoplanetDetail, NumericFieldSummary,
    StableValueSummary,
};

#[component]
pub fn PlanetSummarySection(detail: ExoplanetDetail) -> impl IntoView {
    let canonical = detail.canonical;
    let hostname = canonical.hostname;
    let discovery_method = canonical.discovery_method;
    let discovery_year = canonical.discovery_year;
    let numeric_cards = vec![
        canonical.orbital_period,
        canonical.semi_major_axis,
        canonical.radius,
        canonical.mass,
        canonical.density,
        canonical.equilibrium_temperature,
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    view! {
        <section class="planet-detail-section">
            <div class="planet-detail-section__header">
                <div>
                    <p class="planet-detail-section__eyebrow planet-detail-section__eyebrow--summary">"Canonical Summary"</p>
                    <h2 class="planet-detail-section__title">"Adopted planet values from all rows"</h2>
                </div>
                <p class="planet-detail-section__description">
                    "Numeric fields use the median of non-null measurements. Disagreement stays visible through ranges, counts, and provenance."
                </p>
            </div>

            <div class="planet-summary-grid">
                {hostname.map(|summary| view! {
                    <HostStarSummaryCard summary=summary />
                })}
                {discovery_method.map(|summary| view! {
                    <CategoricalSummaryCard summary=summary />
                })}
                {discovery_year.map(|summary| view! {
                    <StableSummaryCard summary=summary />
                })}
                {numeric_cards.into_iter().map(|summary| view! {
                    <NumericSummaryCard summary=summary />
                }).collect::<Vec<_>>()}
            </div>
        </section>
    }
}

#[component]
fn NumericSummaryCard(summary: NumericFieldSummary) -> impl IntoView {
    let primary = match summary.unit.as_deref() {
        Some(unit) if !unit.is_empty() => {
            format!("{} {unit}", format_number(summary.value))
        }
        _ => format_number(summary.value),
    };
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
        <article class="planet-summary-card">
            <p class="planet-summary-card__label">{summary.label.clone()}</p>
            <p class="planet-summary-card__value">{primary}</p>
            <p class="planet-summary-card__body">{range}</p>
            <p class="planet-summary-card__meta">{evidence}</p>
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
        <article class="planet-summary-card">
            <p class="planet-summary-card__label">{summary.label.clone()}</p>
            <p class="planet-summary-card__value">{summary.value.clone()}</p>
            <p class="planet-summary-card__body">
                {if summary.disputed {
                    "Multiple classifications reported"
                } else {
                    "Consistent across records"
                }}
            </p>
            <p class="planet-summary-card__meta">{counts}</p>
        </article>
    }
}

#[component]
fn StableSummaryCard(summary: StableValueSummary) -> impl IntoView {
    let unit = summary.unit.as_deref().unwrap_or("");
    let value = format_value(&summary.value, unit);
    let distinct = summary
        .distinct_values
        .iter()
        .map(|value| format_value(value, unit))
        .collect::<Vec<_>>()
        .join(", ");

    view! {
        <article class="planet-summary-card">
            <p class="planet-summary-card__label">{summary.label.clone()}</p>
            <p class="planet-summary-card__value">{value}</p>
            <p class="planet-summary-card__body">
                {if summary.disputed {
                    "Values disagree across records"
                } else {
                    "Stable across source rows"
                }}
            </p>
            <p class="planet-summary-card__meta">{distinct}</p>
        </article>
    }
}

#[component]
fn HostStarSummaryCard(summary: StableValueSummary) -> impl IntoView {
    let label = summary.label.clone();
    let value = format_value(&summary.value, "");
    let href = summary
        .value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|host| format!("/stellarhosts/{}", encode_path_segment(host)));
    let body = if summary.disputed {
        "Host names differ across records".to_string()
    } else {
        "Stable across source rows".to_string()
    };
    let distinct = summary
        .distinct_values
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(", ");

    match href {
        Some(href) => view! {
            <A
                href=href
                attr:class="planet-summary-card planet-summary-card--interactive"
            >
                <p class="planet-summary-card__label">{label}</p>
                <p class="planet-summary-card__value">{value}</p>
                <p class="planet-summary-card__body">{body}</p>
                <p class="planet-summary-card__meta">{distinct}</p>
                <span class="planet-summary-card__arrow" aria-hidden="true">"→"</span>
            </A>
        }
        .into_any(),
        None => view! {
            <article class="planet-summary-card">
                <p class="planet-summary-card__label">{label}</p>
                <p class="planet-summary-card__value">{value}</p>
                <p class="planet-summary-card__body">{body}</p>
                <p class="planet-summary-card__meta">{distinct}</p>
            </article>
        }
        .into_any(),
    }
}
