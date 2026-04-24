use leptos::prelude::*;
use leptos::serde_json::Value;
use leptos::server_fn::ServerFnError;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

use crate::metadata_helpers::{encode_path_segment, title_with_site};
use crate::server::functions::TableData;

const HOST_LINK_COLUMN: &str = "host_link_hostname";

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
    let facts = insight_facts(&render_columns, &data.rows);
    let rows = data.rows;

    view! {
        <div class="insight-data-panel">
            <InsightFacts facts=facts/>
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
                        {rows.into_iter().enumerate().map(|(idx, row)| {
                            view! { <InsightDataRow rank=idx + 1 columns=render_columns.clone() row=row/> }
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn InsightFacts(facts: Vec<String>) -> impl IntoView {
    view! {
        <div class="insight-data-panel__facts">
            <p class="insight-data-panel__facts-label">"What this view shows"</p>
            <div class="insight-data-panel__facts-list">
                {facts.into_iter().map(|fact| view! {
                    <p class="insight-data-panel__fact">{fact}</p>
                }).collect::<Vec<_>>()}
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
    columns
        .iter()
        .filter(|column| !is_link_helper_column(column))
        .cloned()
        .collect()
}

fn is_link_helper_column(column: &str) -> bool {
    column == HOST_LINK_COLUMN
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
                .get(HOST_LINK_COLUMN)
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
        "pl_orbsmax" => "Semi-Major Axis",
        "pl_orbper" => "Period",
        "pl_host_radius_ratio" => "Size Ratio",
        "pl_host_radius_ratio_delta" => "Gap From Equal",
        "pl_bmasse" => "Mass",
        "disc_year" => "Year",
        "st_teff" => "Teff",
        "st_mass" => "Mass",
        "st_rad" => "Host Radius",
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
                "pl_orbsmax" => " AU",
                "pl_orbper" => " d",
                "pl_host_radius_ratio" => "x",
                "pl_host_radius_ratio_delta" => "",
                "pl_bmasse" => " M⊕",
                "st_teff" => " K",
                "st_rad" => " R☉",
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

fn insight_facts(columns: &[String], rows: &[Value]) -> Vec<String> {
    let mut facts = Vec::new();

    if let Some(column) = primary_metric_column(columns)
        && let Some(range) = numeric_range(rows, column)
    {
        facts.push(format!(
            "{} spans {} to {} across the ranked entries.",
            label_for_column(column),
            format_numeric_value(column, range.min),
            format_numeric_value(column, range.max)
        ));
    }

    if let Some(year_range) = numeric_range(rows, "disc_year") {
        if (year_range.min - year_range.max).abs() < f64::EPSILON {
            facts.push(format!(
                "Discovery year is {} for the ranked entries with year data.",
                format_numeric_value("disc_year", year_range.min)
            ));
        } else {
            facts.push(format!(
                "Discovery years run from {} to {}.",
                format_numeric_value("disc_year", year_range.min),
                format_numeric_value("disc_year", year_range.max)
            ));
        }
    }

    if columns.iter().any(|column| column == "pl_bmasse") {
        let available = count_numeric_values(rows, "pl_bmasse");
        if available == 0 {
            facts.push(
                "Mass is not available for these ranked entries.".to_string(),
            );
        } else if available < rows.len() {
            facts.push(format!(
                "Mass is available for {} of {} ranked entries.",
                available,
                rows.len()
            ));
        }
    }

    if facts.is_empty() {
        facts.push(
            "Rows are ordered by the insight-specific metric using current catalog values."
                .to_string(),
        );
    }

    facts
}

fn primary_metric_column(columns: &[String]) -> Option<&str> {
    [
        "pl_rade",
        "pl_orbsmax",
        "sy_dist",
        "pl_host_radius_ratio",
        "pl_host_radius_ratio_delta",
        "st_teff",
        "sy_pnum",
        "sy_snum",
    ]
    .into_iter()
    .find(|candidate| columns.iter().any(|column| column == candidate))
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct NumericRange {
    min: f64,
    max: f64,
}

fn numeric_range(rows: &[Value], column: &str) -> Option<NumericRange> {
    let mut values = rows
        .iter()
        .filter_map(|row| row.get(column).and_then(Value::as_f64));
    let first = values.next()?;
    let mut min = first;
    let mut max = first;

    for value in values {
        min = min.min(value);
        max = max.max(value);
    }

    Some(NumericRange { min, max })
}

fn count_numeric_values(rows: &[Value], column: &str) -> usize {
    rows.iter()
        .filter(|row| row.get(column).and_then(Value::as_f64).is_some())
        .count()
}

fn format_numeric_value(column: &str, value: f64) -> String {
    let value = Value::from(value);
    format_cell(column, &value)
}

#[cfg(test)]
mod tests {
    use leptos::serde_json::json;

    use super::{
        HOST_LINK_COLUMN, NumericRange, href_for_column, insight_facts,
        numeric_range, render_columns,
    };

    #[test]
    fn render_columns_hides_explicit_link_helper_columns() {
        let columns = vec![
            "sy_name".to_string(),
            HOST_LINK_COLUMN.to_string(),
            "sy_pnum".to_string(),
        ];

        assert_eq!(
            render_columns(&columns),
            vec!["sy_name".to_string(), "sy_pnum".to_string()]
        );
    }

    #[test]
    fn system_name_links_use_explicit_host_link_column() {
        let row = json!({
            "sy_name": "Kepler-90",
            HOST_LINK_COLUMN: "Kepler-90",
        });
        let href = href_for_column("sy_name", &json!("Kepler-90"), &row);

        assert_eq!(href.as_deref(), Some("/stellarhosts/Kepler%2D90"));
    }

    #[test]
    fn system_name_does_not_fall_back_to_display_hostname() {
        let row = json!({
            "sy_name": "Kepler-90",
            "hostname": "Kepler-90",
        });
        let href = href_for_column("sy_name", &json!("Kepler-90"), &row);

        assert_eq!(href, None);
    }

    #[test]
    fn numeric_range_ignores_missing_values() {
        let rows = vec![
            json!({ "pl_rade": 0.31 }),
            json!({ "pl_rade": null }),
            json!({ "pl_rade": 0.50 }),
        ];

        assert_eq!(
            numeric_range(&rows, "pl_rade"),
            Some(NumericRange {
                min: 0.31,
                max: 0.50
            })
        );
    }

    #[test]
    fn insight_facts_describe_metric_range_and_missing_mass() {
        let columns = vec![
            "pl_name".to_string(),
            "pl_rade".to_string(),
            "pl_bmasse".to_string(),
            "disc_year".to_string(),
        ];
        let rows = vec![
            json!({ "pl_rade": 0.31, "pl_bmasse": 0.79, "disc_year": 2013 }),
            json!({ "pl_rade": 0.50, "pl_bmasse": null, "disc_year": 2024 }),
        ];

        let facts = insight_facts(&columns, &rows);

        assert_eq!(
            facts,
            vec![
                "Radius spans 0.31 R⊕ to 0.50 R⊕ across the ranked entries.",
                "Discovery years run from 2013 to 2024.",
                "Mass is available for 1 of 2 ranked entries.",
            ]
        );
    }
}
