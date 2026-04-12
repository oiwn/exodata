use std::collections::{BTreeSet, HashMap};

use leptos::prelude::*;
use leptos::serde_json::Value;

use super::format::{first_non_empty_string, format_value};
use crate::server::functions::{ColumnMetadata, ExoplanetDetail};

const PROVENANCE_COLUMNS: &[&str] = &[
    "disc_year",
    "discoverymethod",
    "disc_facility",
    "disc_telescope",
    "pl_orbper",
    "pl_rade",
    "pl_bmasse",
    "pl_eqt",
    "pl_refname",
];

const SUMMARY_STAT_KEYS: &[(&str, &str)] = &[
    ("discoverymethod", "Discovery"),
    ("disc_facility", "Facility"),
    ("pl_orbper", "Period"),
    ("pl_rade", "Radius"),
    ("pl_bmasse", "Mass"),
    ("pl_eqt", "Eq. Temp"),
];

#[component]
pub fn PlanetRecordsSection(detail: ExoplanetDetail) -> impl IntoView {
    let rows = detail.records.len();
    let ref_labels = collect_refs(&detail.records);
    let discovery_methods =
        distinct_non_empty_count(&detail.records, "discoverymethod");
    let columns = available_columns(&detail.records, &detail.metadata);
    let stats = build_stats(&detail.records);

    view! {
        <section class="planet-detail-section">
            <div class="planet-detail-section__header">
                <div>
                    <p class="planet-detail-section__eyebrow planet-detail-section__eyebrow--records">"Provenance"</p>
                    <h2 class="planet-detail-section__title">"Underlying measurements and references"</h2>
                </div>
                <div class="planet-provenance__actions">
                    <button class="planet-provenance__action" disabled>"Download JSON"</button>
                    <button class="planet-provenance__action" disabled>"Download CSV"</button>
                </div>
            </div>

            <div class="planet-provenance__layout">
                <div class="planet-provenance__panel planet-provenance__panel--summary">
                    <p class="planet-summary-card__label">"Evidence Summary"</p>
                    <div class="planet-provenance__summary-grid">
                        <ProvenanceMetric label="Rows" value=rows.to_string() />
                        <ProvenanceMetric
                            label="Refs"
                            value=ref_labels.len().to_string()
                        />
                        <ProvenanceMetric
                            label="Methods"
                            value=discovery_methods.to_string()
                        />
                    </div>
                    <div class="planet-provenance__stats">
                        {stats.into_iter().map(|stat| view! {
                            <div class="planet-provenance__stat">
                                <div class="planet-provenance__stat-row">
                                    <span class="planet-provenance__stat-label">{stat.label}</span>
                                    <span class="planet-provenance__stat-status">
                                        {if stat.distinct_count > 1 { "disputed" } else { "stable" }}
                                    </span>
                                </div>
                                <p class="planet-provenance__stat-meta">
                                    {format!("{} values • {} distinct", stat.value_count, stat.distinct_count)}
                                </p>
                            </div>
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                <div class="planet-provenance__panel planet-provenance__panel--table">
                    <div class="planet-provenance__table-wrap">
                        <table class="planet-provenance__table">
                            <thead>
                                <tr>
                                    {columns.iter().map(|column| view! {
                                        <th class="planet-provenance__table-head">
                                            {column_label(column)}
                                        </th>
                                    }).collect::<Vec<_>>()}
                                </tr>
                            </thead>
                            <tbody>
                                {detail.records.into_iter().map(|record| {
                                    let columns = columns.clone();
                                    let metadata = detail.metadata.clone();
                                    view! {
                                        <tr class="planet-provenance__table-row">
                                            {columns.into_iter().map(|column| {
                                                let value = record.get(&column).cloned().unwrap_or(Value::Null);
                                                let unit = metadata
                                                    .get(&column)
                                                    .and_then(|item| item.unit.clone())
                                                    .unwrap_or_default();

                                                view! {
                                                    <td class="planet-provenance__table-cell">
                                                        <ProvenanceCell column=column value=value unit=unit />
                                                    </td>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn ProvenanceMetric(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="planet-provenance-metric">
            <p class="planet-provenance-metric__label">{label}</p>
            <p class="planet-provenance-metric__value">{value}</p>
        </div>
    }
}

#[component]
fn ProvenanceCell(column: String, value: Value, unit: String) -> impl IntoView {
    let is_ref_column = matches!(column.as_str(), "pl_refname");

    if is_ref_column && let Some(link) = parse_archive_anchor(&value) {
        return view! {
            <a
                class="planet-provenance__ref-link"
                href=link.href
                target="_blank"
                rel="nofollow noopener noreferrer"
            >
                {link.label}
            </a>
        }
        .into_any();
    }

    if column == "disc_telescope" && matches!(value, Value::Null) {
        return view! {
            <span class="planet-provenance__cell-value">
                "—"
            </span>
        }
        .into_any();
    }

    view! {
        <span class="planet-provenance__cell-value">{format_value(&value, &unit)}</span>
    }
    .into_any()
}

#[derive(Clone)]
struct FieldStat {
    label: &'static str,
    value_count: usize,
    distinct_count: usize,
}

fn build_stats(records: &[Value]) -> Vec<FieldStat> {
    SUMMARY_STAT_KEYS
        .iter()
        .filter_map(|(key, label)| {
            let values = non_empty_values(records, key);
            if values.is_empty() {
                None
            } else {
                let distinct_count = values.iter().collect::<BTreeSet<_>>().len();
                Some(FieldStat {
                    label,
                    value_count: values.len(),
                    distinct_count,
                })
            }
        })
        .collect()
}

fn available_columns(
    records: &[Value],
    metadata: &HashMap<String, ColumnMetadata>,
) -> Vec<String> {
    PROVENANCE_COLUMNS
        .iter()
        .filter(|key| metadata.contains_key(**key) || has_any_value(records, key))
        .map(|key| (*key).to_string())
        .collect()
}

fn column_label(key: &str) -> String {
    match key {
        "disc_year" => "Year".to_string(),
        "discoverymethod" => "Method".to_string(),
        "disc_facility" => "Facility".to_string(),
        "disc_telescope" => "Telescope".to_string(),
        "pl_orbper" => "Period".to_string(),
        "pl_rade" => "Radius".to_string(),
        "pl_bmasse" => "Mass".to_string(),
        "pl_eqt" => "Eq. Temp".to_string(),
        "pl_refname" => "Reference".to_string(),
        _ => key.replace('_', " ").to_uppercase(),
    }
}

fn collect_refs(records: &[Value]) -> Vec<String> {
    let mut refs = records
        .iter()
        .filter_map(|record| {
            record
                .get("pl_refname")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .or_else(|| {
                    first_non_empty_string(
                        std::slice::from_ref(record),
                        "disc_refname",
                    )
                })
        })
        .collect::<Vec<_>>();

    refs.sort();
    refs.dedup();
    refs
}

fn distinct_non_empty_count(records: &[Value], key: &str) -> usize {
    non_empty_values(records, key)
        .into_iter()
        .collect::<BTreeSet<_>>()
        .len()
}

fn non_empty_values(records: &[Value], key: &str) -> Vec<String> {
    records
        .iter()
        .filter_map(|record| {
            record.get(key).and_then(|value| match value {
                Value::Null => None,
                Value::String(text) => {
                    let trimmed = text.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                }
                _ => Some(format_value(value, "")),
            })
        })
        .collect()
}

fn has_any_value(records: &[Value], key: &str) -> bool {
    records.iter().any(|record| {
        record.get(key).is_some_and(|value| match value {
            Value::Null => false,
            Value::String(text) => !text.trim().is_empty(),
            _ => true,
        })
    })
}

struct ParsedArchiveAnchor {
    href: String,
    label: String,
}

fn parse_archive_anchor(value: &Value) -> Option<ParsedArchiveAnchor> {
    let raw = value.as_str()?.trim();
    if !raw.starts_with("<a ") {
        return None;
    }

    let href = extract_attr(raw, "href")?;
    let label = raw
        .split_once('>')
        .and_then(|(_, rest)| rest.rsplit_once("</a>").map(|(label, _)| label))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .unwrap_or(&href)
        .to_string();

    Some(ParsedArchiveAnchor { href, label })
}

fn extract_attr(raw: &str, attr: &str) -> Option<String> {
    let needle = format!("{attr}=");
    let (_, rest) = raw.split_once(&needle)?;
    let quote = rest.chars().next()?;

    if quote == '"' || quote == '\'' {
        let rest = &rest[1..];
        let end = rest.find(quote)?;
        Some(rest[..end].to_string())
    } else {
        let end = rest.find([' ', '>']).unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }
}
