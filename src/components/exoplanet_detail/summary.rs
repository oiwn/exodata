use leptos::prelude::*;
use leptos::serde_json::Value;

use super::format::{format_value, property_display};
use crate::server::functions::ExoplanetDetail;

#[component]
pub fn PlanetSummarySection(detail: ExoplanetDetail) -> impl IntoView {
    let record = detail.records.first().cloned().unwrap_or(Value::Null);
    let metadata = detail.metadata.clone();

    let key_properties = [
        ("hostname", "Host Star", "", "planet-summary-card--blue"),
        (
            "discoverymethod",
            "Discovery",
            "",
            "planet-summary-card--indigo",
        ),
        ("disc_year", "Year", "", "planet-summary-card--rose"),
        ("pl_orbper", "Period", "days", "planet-summary-card--amber"),
        ("pl_rade", "Radius", "R⊕", "planet-summary-card--emerald"),
        ("pl_bmasse", "Mass", "M⊕", "planet-summary-card--orange"),
        ("pl_eqt", "Eq. Temp", "K", "planet-summary-card--slate"),
    ];

    view! {
        <section class="planet-detail-section">
            <div class="planet-detail-section__header">
                <div>
                    <p class="planet-detail-section__eyebrow planet-detail-section__eyebrow--summary">"Quick Summary"</p>
                    <h2 class="planet-detail-section__title">"Fast scan of the current planet profile"</h2>
                </div>
                <p class="planet-detail-section__description">
                    "This first pass still reads from the currently available detail payload. Canonical adopted summaries will replace first-row values in the next backend pass."
                </p>
            </div>

            <div class="planet-summary-grid">
                {key_properties.iter().map(|(key, label, fallback_unit, modifier)| {
                    let value = property_display(&record, &metadata, key, fallback_unit);
                    let description = metadata
                        .get(*key)
                        .and_then(|meta| meta.description.clone())
                        .unwrap_or_default();

                    view! {
                        <PropertyCard
                            label=label.to_string()
                            value=value
                            description=description
                            modifier=modifier.to_string()
                        />
                    }
                }).collect::<Vec<_>>()}
            </div>
        </section>
    }
}

#[component]
fn PropertyCard(
    label: String,
    value: String,
    description: String,
    modifier: String,
) -> impl IntoView {
    let title = if description.is_empty() {
        label.clone()
    } else {
        description
    };

    view! {
        <article class=format!("planet-summary-card {modifier}") title=title>
            <p class="planet-summary-card__label">{label}</p>
            <p class="planet-summary-card__value">{value}</p>
        </article>
    }
}

#[allow(dead_code)]
fn _format_summary_value(value: &Value, unit: &str) -> String {
    format_value(value, unit)
}
