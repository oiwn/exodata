use leptos::prelude::*;
use leptos::serde_json::Value;
use leptos::server_fn::ServerFnError;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

use crate::metadata_helpers::{encode_path_segment, title_with_site};
use crate::server::functions::TableData;

#[component]
pub fn InsightListPageShell(
    eyebrow: &'static str,
    title: &'static str,
    intro: &'static str,
    description: &'static str,
    resource: Resource<Result<TableData, ServerFnError>>,
    empty_label: &'static str,
) -> impl IntoView {
    view! {
        <Title text=title_with_site(title)/>
        <Meta name="description" content=description/>

        <section class="insight-detail-shell">
            <InsightHeader eyebrow=eyebrow title=title intro=intro/>
            <Suspense fallback=InsightLoadingPanel>
                <InsightResourceState resource=resource empty_label=empty_label/>
            </Suspense>
        </section>
    }
}

#[component]
fn InsightHeader(
    eyebrow: &'static str,
    title: &'static str,
    intro: &'static str,
) -> impl IntoView {
    view! {
        <div class="insight-detail-shell__header">
            <p class="insights-section__eyebrow">{eyebrow}</p>
            <h1 class="insight-detail-shell__title">{title}</h1>
            <p class="insight-detail-shell__intro">{intro}</p>
        </div>
    }
}

#[component]
fn InsightLoadingPanel() -> impl IntoView {
    view! {
        <div class="insight-data-panel insight-data-panel--loading">
            <p class="insight-data-panel__loading-label">"Loading insight data"</p>
        </div>
    }
}

#[component]
fn InsightResourceState(
    resource: Resource<Result<TableData, ServerFnError>>,
    empty_label: &'static str,
) -> impl IntoView {
    view! {
        {move || {
            resource.get().map(|result| match result {
                Ok(data) if data.rows.is_empty() => {
                    view! { <InsightEmptyPanel label=empty_label/> }.into_any()
                }
                Ok(data) => view! { <InsightDataPanel data=data/> }.into_any(),
                Err(error) => {
                    view! { <InsightErrorPanel message=error.to_string()/> }.into_any()
                }
            })
        }}
    }
}

#[component]
fn InsightEmptyPanel(label: &'static str) -> impl IntoView {
    view! {
        <div class="insight-data-panel">
            <p class="insight-data-panel__empty">{label}</p>
        </div>
    }
}

#[component]
fn InsightErrorPanel(message: String) -> impl IntoView {
    view! {
        <div class="insight-data-panel insight-data-panel--error">
            <p class="insight-data-panel__error-title">"Error loading insight"</p>
            <p class="insight-data-panel__error-message">{message}</p>
        </div>
    }
}

#[component]
fn InsightDataPanel(data: TableData) -> impl IntoView {
    let columns = data.columns.clone();
    let render_columns = render_columns(&columns);

    view! {
        <div class="insight-data-panel">
            <div class="insight-data-panel__summary">
                <p class="insight-data-panel__summary-label">"Rows"</p>
                <p class="insight-data-panel__summary-value">{data.rows.len().to_string()}</p>
            </div>
            <div class="insight-data-panel__table-wrap">
                <table class="insight-data-panel__table">
                    <thead>
                        <tr>
                            <th class="insight-data-panel__head">"#"</th>
                            {render_columns.iter().map(|column| view! {
                                <th class="insight-data-panel__head">{label_for_column(column)}</th>
                            }).collect::<Vec<_>>()}
                        </tr>
                    </thead>
                    <tbody>
                        {data.rows.into_iter().enumerate().map(|(idx, row)| {
                            view! { <InsightDataRow rank=idx + 1 columns=render_columns.clone() row=row/> }
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn InsightDataRow(
    rank: usize,
    columns: Vec<String>,
    row: Value,
) -> impl IntoView {
    view! {
        <tr class="insight-data-panel__row">
            <td class="insight-data-panel__cell insight-data-panel__cell--rank">{rank.to_string()}</td>
            {columns.into_iter().map(|column| {
                let value = row.get(&column).cloned().unwrap_or(Value::Null);
                view! { <InsightDataCell column=column value=value row=row.clone()/> }
            }).collect::<Vec<_>>()}
        </tr>
    }
}

#[component]
fn InsightDataCell(column: String, value: Value, row: Value) -> impl IntoView {
    if let Some(href) = href_for_column(&column, &value, &row) {
        view! {
            <td class="insight-data-panel__cell">
                <A href=href attr:class="text-cyan-300 hover:text-cyan-200 hover:underline transition-colors">
                    {format_cell(&column, &value)}
                </A>
            </td>
        }
        .into_any()
    } else {
        view! {
            <td class="insight-data-panel__cell">{format_cell(&column, &value)}</td>
        }
        .into_any()
    }
}

fn render_columns(columns: &[String]) -> Vec<String> {
    let is_system_table = columns.iter().any(|column| column == "sy_name");

    columns
        .iter()
        .filter(|column| !(is_system_table && column.as_str() == "hostname"))
        .cloned()
        .collect()
}

fn href_for_column(column: &str, value: &Value, row: &Value) -> Option<String> {
    match column {
        "pl_name" => {
            let slug = value.as_str().map(encode_path_segment)?;
            Some(format!("/exoplanets/{slug}"))
        }
        "hostname" => {
            let slug = value.as_str().map(encode_path_segment)?;
            Some(format!("/stellarhosts/{slug}"))
        }
        "sy_name" => {
            let slug = row
                .get("hostname")
                .and_then(Value::as_str)
                .filter(|host| !host.trim().is_empty())
                .map(encode_path_segment)?;
            Some(format!("/stellarhosts/{slug}"))
        }
        _ => None,
    }
}

fn label_for_column(column: &str) -> &'static str {
    match column {
        "pl_name" => "Planet",
        "hostname" => "Host Star",
        "sy_name" => "System",
        "pl_rade" => "Radius",
        "pl_bmasse" => "Mass",
        "disc_year" => "Year",
        "st_teff" => "Teff",
        "st_mass" => "Mass",
        "sy_snum" => "Stars",
        "sy_pnum" => "Planets",
        "sy_dist" => "Distance",
        _ => "Value",
    }
}

fn format_cell(column: &str, value: &Value) -> String {
    match value {
        Value::Null => "—".to_string(),
        Value::String(text) => text.clone(),
        Value::Number(number) => {
            let suffix = match column {
                "pl_rade" => " R⊕",
                "pl_bmasse" => " M⊕",
                "st_teff" => " K",
                "sy_dist" => " pc",
                _ => "",
            };

            if let Some(value) = number.as_f64() {
                if value.fract() == 0.0 {
                    format!("{value:.0}{suffix}")
                } else {
                    format!("{value:.2}{suffix}")
                }
            } else {
                format!("{number}{suffix}")
            }
        }
        _ => value.to_string(),
    }
}
