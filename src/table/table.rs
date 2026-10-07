use crate::i18n::*;
use crate::locale::localized_path;
use crate::server::functions::TableData;
use crate::table::ColumnGroup;
use crate::table::TableQueryState;
use exo_types::metadata::ColumnMetadata;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use leptos::serde_json::Value;
use leptos_i18n::td_string;
use leptos_router::components::A;
use std::collections::HashMap;

#[component]
fn MeasurementCell(
    value: String,
    err1: Option<String>,
    err2: Option<String>,
    lim_present: bool,
) -> impl IntoView {
    let lim_class = if lim_present {
        "bg-amber-500/10 border-amber-500/30"
    } else {
        ""
    };

    view! {
        <td class=format!("px-3 py-4 text-sm text-gray-300 font-mono border-l border-transparent {}", lim_class)>
            <div class="flex items-center gap-1">
                <div class="flex flex-col text-[10px] leading-none text-gray-400 min-w-[2.25rem] text-right">
                    {err1.as_ref().map(|v| {
                        view! { <span class="-translate-y-0.5">{"+"}{v.clone()}</span> }
                    })}
                    {err2.as_ref().map(|v| {
                        view! { <span class="translate-y-0.5">{"-"}{v.clone()}</span> }
                    })}
                </div>
                <span>{value}</span>
            </div>
        </td>
    }
}

#[component]
pub fn Table(
    data: TableData,
    on_sort: Callback<String>,
    current_sort_column: Option<String>,
    current_sort_order: String,
    column_metadata: HashMap<String, ColumnMetadata>,
    /// Localize only the eight default exoplanet display headings.
    #[prop(optional)]
    exoplanet_headings: bool,
    #[prop(optional)] column_descriptions: Option<HashMap<String, String>>,
    #[prop(optional)] display_columns: Option<Vec<String>>,
    #[prop(optional)] column_groups: Option<HashMap<String, ColumnGroup>>,
    #[prop(optional)] filter_input: Option<ReadSignal<String>>,
    #[prop(optional)] set_filter_input: Option<WriteSignal<String>>,
    #[prop(optional)] on_filter_commit: Option<Callback<String>>,
    /// Column name to render as a link (e.g., "hostname")
    #[prop(optional)]
    link_column: Option<String>,
    /// Base URL for the link (e.g., "/stellarhosts/") - column value will be appended
    #[prop(optional)]
    link_base: Option<String>,
) -> impl IntoView {
    let columns = display_columns.unwrap_or_else(|| data.columns.clone());
    let i18n = use_i18n();
    let locale = i18n.get_locale_untracked();
    let exoplanet_locale = exoplanet_headings.then_some(locale);
    let link_base = link_base.map(|base| localized_path(&base, locale));
    let groups = column_groups.unwrap_or_default();
    let show_filter = filter_input.is_some()
        && set_filter_input.is_some()
        && on_filter_commit.is_some();

    view! {
        <div class="overflow-x-auto rounded-xl border border-slate-700 bg-slate-800/50 backdrop-blur-sm">
            <table class="w-full whitespace-nowrap border-collapse">
                <thead class="bg-slate-900/50 sticky top-0">
                    <tr>
                        {columns.iter().enumerate().map(|(column_idx, col)| {
                            let col_name = col.clone();
                            let col_display = format_column_name(&col_name, exoplanet_locale);
                            let is_sorted = current_sort_column.as_ref() == Some(&col_name);
                            let sort_indicator = if is_sorted {
                                if current_sort_order == "asc" {
                                    " ↑"
                                } else {
                                    " ↓"
                                }
                            } else {
                                ""
                            };

                            // Get description for tooltip
                            let description = column_descriptions
                                .as_ref()
                                .and_then(|descs| descs.get(&col_name))
                                .cloned()
                                .or_else(|| {
                                    exoplanet_locale
                                        .and_then(|locale| default_exoplanet_description(&col_name, locale))
                                        .map(str::to_owned)
                                })
                                .or_else(|| {
                                    column_metadata
                                        .get(&col_name)
                                        .and_then(|m| m.description.clone())
                                });
                            let unit = column_metadata
                                .get(&col_name)
                                .and_then(|m| m.unit.clone());
                            let title = build_column_title(description.clone(), unit)
                                .map(|title| format!("{col_name}: {title}"));
                            let tooltip_id = title.as_ref().map(|_| format!("catalog-column-{col_name}-tooltip"));
                            let tooltip_describedby = tooltip_id.clone();
                            let tooltip_class = if column_idx >= columns.len() / 2 {
                                "catalog-table-tooltip catalog-table-tooltip--right"
                            } else {
                                "catalog-table-tooltip"
                            };

                            let col_for_click = col_name.clone();
                            let col_for_key = col_name.clone();
                            let on_click = move |_| {
                                on_sort.run(col_for_click.clone());
                            };

                            view! {
                                <th
                                    class="catalog-table-heading px-3 py-3 border-r border-b border-slate-700/60 last:border-r-0 text-left text-xs font-semibold text-gray-300 cursor-pointer hover:text-white hover:bg-slate-800/50 transition-colors select-none group relative"
                                    on:click=on_click
                                    tabindex="0"
                                    aria-describedby=tooltip_describedby
                                    on:keydown=move |event: KeyboardEvent| {
                                        if event.key() == "Enter" || event.key() == " " {
                                            event.prevent_default();
                                            on_sort.run(col_for_key.clone());
                                        }
                                    }
                                >
                                    <div class="flex items-center gap-1">
                                        <span class="whitespace-nowrap">{col_display}</span>
                                        {if is_sorted {
                                            view! { <span class="text-purple-400">{sort_indicator}</span> }.into_any()
                                        } else {
                                            view! { <span class="text-gray-600 opacity-0 group-hover:opacity-100">"↕"</span> }.into_any()
                                        }}
                                        {if description.is_some() {
                                            view! { <span class="text-purple-400/60 text-xs">{"ℹ"}</span> }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                    {title.map(|text| view! {
                                        <span id=tooltip_id role="tooltip" class=tooltip_class>{text}</span>
                                    })}
                                </th>
                            }
                        }).collect::<Vec<_>>()}
                    </tr>
                    {if show_filter {
                        let filter_input = filter_input.unwrap();
                        let set_filter_input = set_filter_input.unwrap();
                        let on_filter_commit = on_filter_commit.unwrap();
                        let commit = move || {
                            let value = filter_input.get().trim().to_string();
                            on_filter_commit.run(value);
                        };
                        view! {
                            <tr>
                                {columns.iter().enumerate().map(|(idx, _)| {
                                    if idx == 0 {
                                        view! {
                                            <th class="px-3 py-2 border-r border-b border-slate-700/60 last:border-r-0">
                                                <input
                                                    type="text"
                                                    class="w-32 max-w-full px-2 py-2 rounded-md bg-slate-900/50 border border-slate-700 text-gray-200 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent text-sm"
                                                    placeholder=t_string!(i18n, table_controls.filter)
                                                    prop:value=move || filter_input.get()
                                                    on:input=move |e| {
                                                        set_filter_input.set(event_target_value(&e));
                                                    }
                                                    on:blur=move |_| commit()
                                                    on:keydown=move |e: KeyboardEvent| {
                                                        if e.key() == "Enter" {
                                                            commit();
                                                        }
                                                    }
                                                />
                                            </th>
                                        }.into_any()
                                    } else {
                                        view! { <th class="px-3 py-2 border-r border-b border-slate-700/60 last:border-r-0"></th> }.into_any()
                                    }
                                }).collect::<Vec<_>>()}
                            </tr>
                        }.into_any()
                    } else {
                        ().into_any()
                    }}
                </thead>
                <tbody>
                    {data.rows.iter().enumerate().map(|(idx, row)| {
                        let row_class = if idx % 2 == 0 {
                            "bg-slate-800/30"
                        } else {
                            "bg-slate-800/10"
                        };

                        let link_col = link_column.clone();
                        let link_url_base = link_base.clone();

                        view! {
                            <tr class=format!("{} hover:bg-slate-700/50 transition-colors", row_class)>
                                {columns.iter().map(|col| {
                                    let value = row.get(col).unwrap_or(&Value::Null);
                                    let formatted_value = format_display_cell_value(col, value);
                                    let is_link_column = link_col.as_ref() == Some(col);
                                    let group = groups.get(col);

                                    if is_link_column {
                                        let link_value = value.as_str().unwrap_or("");
                                        // Simple URL encoding for the most common cases
                                        let encoded = link_value.replace(' ', "%20").replace('#', "%23");
                                        let href = link_url_base.as_ref()
                                            .map(|base| format!("{}{}", base, encoded))
                                            .unwrap_or_default();

                                        view! {
                                            <td class="px-3 py-4 text-sm font-mono">
                                                <A
                                                    href=href
                                                    attr:class="text-purple-400 hover:text-purple-300 hover:underline transition-colors"
                                                >
                                                    {formatted_value}
                                                </A>
                                            </td>
                                        }.into_any()
                                    } else if let Some(group) = group {
                                        let base_value = row.get(&group.base).unwrap_or(&Value::Null);
                                        let err1_value = group.err1.as_ref().and_then(|c| row.get(c));
                                        let err2_value = group.err2.as_ref().and_then(|c| row.get(c));
                                        let lim_value = group.lim.as_ref().and_then(|c| row.get(c));
                                        let err1 = err1_value.and_then(format_error_value);
                                        let err2 = err2_value.and_then(format_error_value);
                                        let lim_present = lim_value.map(is_value_present).unwrap_or(false);
                                        let base_formatted = format_cell_value(base_value);

                                        view! {
                                            <MeasurementCell
                                                value=base_formatted
                                                err1=err1
                                                err2=err2
                                                lim_present=lim_present
                                            />
                                        }.into_any()
                                    } else {
                                        let full_method = (col == "discoverymethod"
                                            && value.as_str() == Some("Transit Timing Variations"))
                                            .then(|| value.as_str().unwrap().to_string());
                                        let tooltip_id = format!("catalog-method-{idx}-tooltip");
                                        let tooltip_describedby = tooltip_id.clone();
                                        view! {
                                            <td class="px-3 py-4 text-sm text-gray-300 font-mono">
                                                {match full_method {
                                                    Some(full_method) => view! {
                                                        <span class="catalog-table-abbreviation relative inline-block cursor-help" tabindex="0" aria-describedby=tooltip_describedby>
                                                            {formatted_value}
                                                            <span id=tooltip_id role="tooltip" class="catalog-table-tooltip">{full_method}</span>
                                                        </span>
                                                    }.into_any(),
                                                    None => formatted_value.into_any(),
                                                }}
                                            </td>
                                        }.into_any()
                                    }
                                }).collect::<Vec<_>>()}
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>

            {if data.rows.is_empty() {
                view! {
                    <div class="text-center py-12 text-gray-400">
                        <div class="text-4xl mb-4">"🌌"</div>
                        <p class="text-lg">{t!(i18n, table_controls.no_data)}</p>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}

/// Build query string for table pagination and sorting
///
/// # Arguments
/// * `page` - Current page number
/// * `sort_col` - Optional column name to sort by
/// * `order` - Sort order ("asc" or "desc")
///
/// # Returns
/// Query string like "page=2&sort=hostname&order=asc"
pub fn build_table_query(query: &TableQueryState) -> String {
    let mut query_params = Vec::new();
    if query.page > 1 {
        query_params.push(format!("page={}", query.page));
    }
    if let Some(col) = query.sort_col.as_deref() {
        query_params.push(format!("sort={}", col));
        query_params.push(format!("order={}", query.sort_order));
    }
    if !query.columns.is_empty() {
        let encoded = encode_query_value(&query.columns.join(","));
        query_params.push(format!("columns={}", encoded));
    }
    let trimmed = query.filter.trim();
    if !trimmed.is_empty() {
        let encoded = encode_query_value(trimmed);
        query_params.push(format!("filter={}", encoded));
    }
    query_params.join("&")
}

/// Format column name for display
fn format_column_name(col: &str, exoplanet_locale: Option<Locale>) -> String {
    if let Some(locale) = exoplanet_locale {
        let label = match col {
            "pl_name" => Some(td_string!(locale, exoplanet_columns.pl_name)),
            "hostname" => Some(td_string!(locale, exoplanet_columns.hostname)),
            "discoverymethod" => {
                Some(td_string!(locale, exoplanet_columns.discoverymethod))
            }
            "disc_year" => Some(td_string!(locale, exoplanet_columns.disc_year)),
            "pl_orbper" => Some(td_string!(locale, exoplanet_columns.pl_orbper)),
            "pl_rade" => Some(td_string!(locale, exoplanet_columns.pl_rade)),
            "pl_bmasse" => Some(td_string!(locale, exoplanet_columns.pl_bmasse)),
            "rowupdate" => Some(td_string!(locale, exoplanet_columns.rowupdate)),
            _ => None,
        };
        if let Some(label) = label {
            return label.to_string();
        }
    }

    match col {
        "hostname" => "Star Name".to_string(),
        "sy_dist" => "Distance (pc)".to_string(),
        "st_teff" => "Temperature (K)".to_string(),
        "st_mass" => "Mass (M☉)".to_string(),
        "sy_pnum" => "Planets".to_string(),
        "rowupdate" => "Updated".to_string(),
        _ => col.to_string(),
    }
}

fn default_exoplanet_description(
    col: &str,
    locale: Locale,
) -> Option<&'static str> {
    match col {
        "pl_name" => {
            Some(td_string!(locale, exoplanet_column_descriptions.pl_name))
        }
        "hostname" => {
            Some(td_string!(locale, exoplanet_column_descriptions.hostname))
        }
        "discoverymethod" => Some(td_string!(
            locale,
            exoplanet_column_descriptions.discoverymethod
        )),
        "disc_year" => {
            Some(td_string!(locale, exoplanet_column_descriptions.disc_year))
        }
        "pl_orbper" => {
            Some(td_string!(locale, exoplanet_column_descriptions.pl_orbper))
        }
        "pl_rade" => {
            Some(td_string!(locale, exoplanet_column_descriptions.pl_rade))
        }
        "pl_bmasse" => {
            Some(td_string!(locale, exoplanet_column_descriptions.pl_bmasse))
        }
        "rowupdate" => {
            Some(td_string!(locale, exoplanet_column_descriptions.rowupdate))
        }
        _ => None,
    }
}

fn build_column_title(
    description: Option<String>,
    unit: Option<String>,
) -> Option<String> {
    match (description, unit) {
        (Some(desc), Some(unit)) => Some(format!("{} [{}]", desc, unit)),
        (Some(desc), None) => Some(desc),
        (None, Some(unit)) => Some(format!("[{}]", unit)),
        (None, None) => None,
    }
}

fn is_value_present(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(s) => !s.trim().is_empty(),
        _ => true,
    }
}

fn encode_query_value(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace(' ', "%20")
        .replace('#', "%23")
        .replace('&', "%26")
        .replace('?', "%3F")
}

fn format_error_value(value: &Value) -> Option<String> {
    let formatted = format_cell_value(value);
    if formatted == "—" {
        return None;
    }
    let trimmed = formatted
        .trim_start_matches('-')
        .trim_start_matches('+')
        .to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Format cell value for display
fn format_display_cell_value(column: &str, value: &Value) -> String {
    if column == "discoverymethod"
        && value.as_str() == Some("Transit Timing Variations")
    {
        "TTV".to_string()
    } else {
        format_cell_value(value)
    }
}

fn format_cell_value(value: &Value) -> String {
    match value {
        Value::Null => "—".to_string(),
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 {
                    format!("{:.0}", f)
                } else {
                    format!("{:.2}", f)
                }
            } else if let Some(i) = n.as_i64() {
                i.to_string()
            } else {
                n.to_string()
            }
        }
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_method_abbreviation_is_display_only() {
        let value = Value::String("Transit Timing Variations".to_string());
        assert_eq!(format_display_cell_value("discoverymethod", &value), "TTV");
        assert_eq!(
            format_display_cell_value("other_column", &value),
            "Transit Timing Variations"
        );
        assert_eq!(value.as_str(), Some("Transit Timing Variations"));
        assert_eq!(
            format_display_cell_value(
                "discoverymethod",
                &Value::String("Transit".to_string())
            ),
            "Transit"
        );
    }

    #[test]
    fn default_exoplanet_tooltips_work_without_source_descriptions() {
        for locale in [Locale::en, Locale::zh_CN, Locale::ja] {
            for column in [
                "pl_name",
                "hostname",
                "discoverymethod",
                "disc_year",
                "pl_orbper",
                "pl_rade",
                "pl_bmasse",
                "rowupdate",
            ] {
                let description = default_exoplanet_description(column, locale)
                    .expect("default column description");
                assert!(!description.trim().is_empty());
                assert!(
                    build_column_title(Some(description.to_string()), None)
                        .is_some()
                );
            }
            let mass =
                default_exoplanet_description("pl_bmasse", locale).unwrap();
            assert!(mass.contains("M sin i"));
            assert!(mass.contains("pl_bmassprov"));
        }
        assert!(default_exoplanet_description("pl_eqt", Locale::en).is_none());
    }

    #[test]
    fn default_tooltips_retain_metadata_units() {
        let description =
            default_exoplanet_description("pl_rade", Locale::en).unwrap();
        let title = build_column_title(
            Some(description.to_string()),
            Some("Rearth".to_string()),
        )
        .unwrap();
        assert!(title.contains(description));
        assert!(title.ends_with("[Rearth]"));
    }

    #[test]
    fn default_exoplanet_headings_use_the_selected_locale() {
        for column in [
            "pl_name",
            "hostname",
            "discoverymethod",
            "disc_year",
            "pl_orbper",
            "pl_rade",
            "pl_bmasse",
            "rowupdate",
        ] {
            let english = format_column_name(column, Some(Locale::en));
            assert_ne!(english, column);
            for locale in [Locale::zh_CN, Locale::ja] {
                let translated = format_column_name(column, Some(locale));
                assert!(!translated.is_empty());
                assert_ne!(translated, column);
                assert_ne!(translated, english);
            }
        }
        assert_eq!(
            format_column_name("pl_name", Some(Locale::en)),
            "Planet name"
        );
        assert_eq!(
            format_column_name("pl_name", Some(Locale::zh_CN)),
            "行星名称"
        );
        assert_eq!(format_column_name("pl_name", Some(Locale::ja)), "惑星名");
        for locale in [Locale::en, Locale::zh_CN, Locale::ja] {
            assert!(
                format_column_name("pl_bmasse", Some(locale)).contains("M sin i")
            );
        }
    }

    #[test]
    fn other_headings_preserve_scientific_keys_and_existing_aliases() {
        for locale in [Locale::en, Locale::zh_CN, Locale::ja] {
            assert_eq!(format_column_name("pl_eqt", Some(locale)), "pl_eqt");
            assert_eq!(
                format_column_name("future_column", Some(locale)),
                "future_column"
            );
            assert_eq!(
                format_column_name("sy_dist", Some(locale)),
                "Distance (pc)"
            );
        }
        assert_eq!(format_column_name("pl_name", None), "pl_name");
        assert_eq!(format_column_name("hostname", None), "Star Name");
    }
}
