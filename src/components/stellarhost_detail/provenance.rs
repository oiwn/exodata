use leptos::prelude::*;
use leptos::serde_json::Value;

use super::format::{column_label, format_json_value};
use crate::metadata_helpers::encode_path_segment;
use crate::server::functions::StellarHostDetail;

#[component]
pub fn ProvenanceSection(host: StellarHostDetail) -> impl IntoView {
    let stat_rows = host.provenance.key_field_stats.clone();
    let columns = host.provenance_columns.clone();
    let records = host.records.clone();
    let encoded_name = encode_path_segment(&host.hostname);
    let json_href = format!("/stellarhosts/{encoded_name}.json");
    let csv_href = format!("/stellarhosts/{encoded_name}.csv");

    view! {
        <section class="host-detail-section">
            <div class="host-detail-section__header">
                <div>
                    <p class="host-detail-section__eyebrow host-detail-section__eyebrow--provenance">"Provenance"</p>
                    <h2 class="host-detail-section__title">"Underlying measurements and references"</h2>
                </div>
                <div class="host-provenance__actions">
                    <a
                        class="host-provenance__action"
                        href=json_href
                        title="Download full detail data as JSON"
                        download
                        rel="external"
                    >
                        "Download JSON"
                    </a>
                    <a
                        class="host-provenance__action"
                        href=csv_href
                        title="Download matching source rows as CSV"
                        download
                        rel="external"
                    >
                        "Download CSV"
                    </a>
                </div>
            </div>

            <div class="host-provenance__layout">
                <div class="host-provenance__panel host-provenance__panel--summary">
                    <p class="host-detail-card__label">"Evidence Summary"</p>
                    <div class="host-provenance__summary-grid">
                        <ProvenanceMetric label="Rows" value=host.provenance.record_count.to_string() />
                        <ProvenanceMetric
                            label="Stellar refs"
                            value=host.provenance.stellar_refs.len().to_string()
                        />
                        <ProvenanceMetric
                            label="System refs"
                            value=host.provenance.system_refs.len().to_string()
                        />
                    </div>
                    <div class="host-provenance__stats">
                        {stat_rows.into_iter().map(|stat| view! {
                            <div class="host-provenance__stat">
                                <div class="host-provenance__stat-row">
                                    <span class="host-provenance__stat-label">{stat.label}</span>
                                    <span class="host-provenance__stat-status">
                                        {if stat.disputed { "disputed" } else { "stable" }}
                                    </span>
                                </div>
                                <p class="host-provenance__stat-meta">
                                    {format!("{} values • {} distinct", stat.measurement_count, stat.distinct_count)}
                                </p>
                            </div>
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                <div class="host-provenance__panel host-provenance__panel--table">
                    <div class="host-provenance__table-wrap">
                        <table class="host-provenance__table">
                            <thead>
                                <tr>
                                    {columns.iter().map(|column| view! {
                                        <th class="host-provenance__table-head">
                                            {column_label(column)}
                                        </th>
                                    }).collect::<Vec<_>>()}
                                </tr>
                            </thead>
                            <tbody>
                                {records.into_iter().map(|record| {
                                    let columns = columns.clone();
                                    view! {
                                        <tr class="host-provenance__table-row">
                                            {columns.into_iter().map(|column| {
                                                let value = record.get(&column).cloned().unwrap_or(Value::Null);
                                                view! {
                                                    <td class="host-provenance__table-cell">
                                                        <ProvenanceCell column=column value=value />
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
        <div class="host-provenance-metric">
            <p class="host-provenance-metric__label">{label}</p>
            <p class="host-provenance-metric__value">{value}</p>
        </div>
    }
}

#[component]
fn ProvenanceCell(column: String, value: Value) -> impl IntoView {
    let is_ref_column = matches!(column.as_str(), "st_refname" | "sy_refname");

    if is_ref_column && let Some(link) = parse_archive_anchor(&value) {
        return view! {
            <a
                class="host-provenance__ref-link"
                href=link.href
                target="_blank"
                rel="nofollow noopener noreferrer"
            >
                {link.label}
            </a>
        }
        .into_any();
    }

    view! {
        <span class="host-provenance__cell-value">{format_json_value(&value, "")}</span>
    }
    .into_any()
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
