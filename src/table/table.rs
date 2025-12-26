use leptos::prelude::*;
use leptos::serde_json::Value;
use crate::server::functions::TableData;
use std::collections::HashMap;

#[component]
pub fn Table(
    data: TableData,
    on_sort: Callback<String>,
    current_sort_column: Option<String>,
    current_sort_order: String,
    #[prop(optional)] column_descriptions: Option<HashMap<String, String>>,
) -> impl IntoView {
    view! {
        <div class="overflow-x-auto rounded-xl border border-slate-700 bg-slate-800/50 backdrop-blur-sm">
            <table class="w-full border-collapse">
                <thead class="bg-slate-900/50 sticky top-0">
                    <tr>
                        {data.columns.iter().map(|col| {
                            let col_name = col.clone();
                            let col_display = format_column_name(&col_name);
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
                            let description = column_descriptions.as_ref()
                                .and_then(|descs| descs.get(&col_name))
                                .map(|s| s.clone());

                            let col_for_click = col_name.clone();
                            let on_click = move |_| {
                                on_sort.run(col_for_click.clone());
                            };

                            view! {
                                <th
                                    class="px-6 py-4 text-left text-xs font-semibold text-gray-300 uppercase tracking-wider cursor-pointer hover:text-white hover:bg-slate-800/50 transition-colors select-none group relative"
                                    on:click=on_click
                                    title=description.clone()
                                >
                                    <div class="flex items-center gap-2">
                                        <span>{col_display}</span>
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
                                </th>
                            }
                        }).collect::<Vec<_>>()}
                    </tr>
                </thead>
                <tbody>
                    {data.rows.iter().enumerate().map(|(idx, row)| {
                        let row_class = if idx % 2 == 0 {
                            "bg-slate-800/30"
                        } else {
                            "bg-slate-800/10"
                        };

                        view! {
                            <tr class=format!("{} hover:bg-slate-700/50 transition-colors", row_class)>
                                {data.columns.iter().map(|col| {
                                    let value = row.get(col).unwrap_or(&Value::Null);
                                    let formatted_value = format_cell_value(value);

                                    view! {
                                        <td class="px-6 py-4 text-sm text-gray-300 font-mono">
                                            {formatted_value}
                                        </td>
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
                        <p class="text-lg">"No data available"</p>
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
/// Query string like "page=1&sort=hostname&order=asc"
pub fn build_table_query(page: usize, sort_col: Option<&str>, order: &str) -> String {
    let mut query_params = vec![format!("page={}", page)];
    if let Some(col) = sort_col {
        query_params.push(format!("sort={}", col));
        query_params.push(format!("order={}", order));
    }
    query_params.join("&")
}

/// Format column name for display
fn format_column_name(col: &str) -> String {
    match col {
        "hostname" => "Star Name".to_string(),
        "sy_dist" => "Distance (pc)".to_string(),
        "st_teff" => "Temperature (K)".to_string(),
        "st_mass" => "Mass (M☉)".to_string(),
        "sy_pnum" => "Planets".to_string(),
        _ => col.to_string(),
    }
}

/// Format cell value for display
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
