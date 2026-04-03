use leptos::prelude::*;
use leptos::serde_json::Value;

use super::format::{column_label, format_json_value};
use crate::server::functions::StellarHostDetail;

#[component]
pub fn ProvenanceSection(host: StellarHostDetail) -> impl IntoView {
    let stat_rows = host.provenance.key_field_stats.clone();
    let columns = host.provenance_columns.clone();
    let records = host.records.clone();

    view! {
        <section class="space-y-6">
            <div class="flex items-end justify-between gap-4">
                <div>
                    <p class="text-sm uppercase tracking-[0.18em] text-emerald-300">"Provenance"</p>
                    <h2 class="mt-2 text-3xl font-semibold text-white">"Underlying measurements and references"</h2>
                </div>
                <div class="flex gap-3">
                    <button class="rounded-full border border-slate-700 px-4 py-2 text-sm text-slate-300">"Download JSON"</button>
                    <button class="rounded-full border border-slate-700 px-4 py-2 text-sm text-slate-300">"Download CSV"</button>
                </div>
            </div>

            <div class="grid gap-4 xl:grid-cols-[1.25fr_3fr]">
                <div class="rounded-[1.5rem] border border-slate-800 bg-slate-950/70 p-6 shadow-lg shadow-slate-950/30">
                    <p class="text-xs uppercase tracking-[0.18em] text-slate-400">"Evidence Summary"</p>
                    <div class="mt-4 grid gap-3 sm:grid-cols-3 xl:grid-cols-3">
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
                    <div class="mt-5 grid gap-3">
                        {stat_rows.into_iter().map(|stat| view! {
                            <div class="rounded-2xl bg-slate-900/80 px-4 py-3">
                                <div class="flex items-start justify-between gap-4">
                                    <span class="text-sm text-slate-200">{stat.label}</span>
                                    <span class="text-xs uppercase tracking-[0.16em] text-slate-500">
                                        {if stat.disputed { "disputed" } else { "stable" }}
                                    </span>
                                </div>
                                <p class="mt-2 text-sm text-slate-400">
                                    {format!("{} values • {} distinct", stat.measurement_count, stat.distinct_count)}
                                </p>
                            </div>
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                <div class="rounded-[1.5rem] border border-slate-800 bg-slate-950/70 p-5 shadow-lg shadow-slate-950/30">
                    <div class="overflow-x-auto">
                        <table class="min-w-full border-separate border-spacing-y-2">
                            <thead>
                                <tr>
                                    {columns.iter().map(|column| view! {
                                        <th class="px-3 py-2 text-left text-xs uppercase tracking-[0.18em] text-slate-500">
                                            {column_label(column)}
                                        </th>
                                    }).collect::<Vec<_>>()}
                                </tr>
                            </thead>
                            <tbody>
                                {records.into_iter().map(|record| {
                                    let columns = columns.clone();
                                    view! {
                                        <tr class="bg-slate-900/80">
                                            {columns.into_iter().map(|column| {
                                                let value = record.get(&column).cloned().unwrap_or(Value::Null);
                                                view! {
                                                    <td class="px-3 py-3 text-sm text-slate-200">
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
        <div class="rounded-2xl bg-slate-900/80 px-4 py-4">
            <p class="text-xs uppercase tracking-[0.16em] text-slate-500">{label}</p>
            <p class="mt-2 text-2xl font-semibold text-white">{value}</p>
        </div>
    }
}

#[component]
fn ProvenanceCell(column: String, value: Value) -> impl IntoView {
    let is_ref_column = matches!(column.as_str(), "st_refname" | "sy_refname");

    if is_ref_column {
        if let Some(link) = parse_archive_anchor(&value) {
            return view! {
                <a
                    class="break-words text-sky-300 underline decoration-sky-400/40 underline-offset-4 hover:text-sky-200"
                    href=link.href
                    target="_blank"
                    rel="nofollow noopener noreferrer"
                >
                    {link.label}
                </a>
            }
            .into_any();
        }
    }

    view! {
        <span class="break-words">{format_json_value(&value, "")}</span>
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
