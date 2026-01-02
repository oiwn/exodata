use crate::server::functions::ColumnMetadata;
use leptos::ev::Event;
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn ColumnSelector(
    /// All available columns with their metadata
    available_columns: ReadSignal<HashMap<String, ColumnMetadata>>,
    /// Currently selected column names
    selected_columns: ReadSignal<Vec<String>>,
    /// Callback when selection changes
    on_change: Callback<Vec<String>>,
) -> impl IntoView {
    let (search_term, set_search_term) = signal(String::new());
    let (is_open, set_is_open) = signal(false);

    // Sort columns alphabetically
    let sorted_columns = move || {
        let mut cols: Vec<_> = available_columns.get().into_iter().collect();
        cols.sort_by(|a, b| a.0.cmp(&b.0));
        cols
    };

    // Filter columns based on search
    let filtered_columns = move || {
        let search = search_term.get().to_lowercase();
        sorted_columns()
            .into_iter()
            .filter(|(name, meta)| {
                if search.is_empty() {
                    return true;
                }
                name.to_lowercase().contains(&search)
                    || meta
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&search))
                        .unwrap_or(false)
            })
            .collect::<Vec<_>>()
    };

    // Select all handler
    let on_select_all = move |_| {
        let all: Vec<String> = available_columns.get().keys().cloned().collect();
        on_change.run(all);
    };

    // Deselect all handler
    let on_deselect_all = move |_| {
        on_change.run(vec![]);
    };

    // Add column from the available list (auto-collapse)
    let add_column = move |column_name: String| {
        let mut current = selected_columns.get();
        if current.contains(&column_name) {
            current.retain(|c| c != &column_name);
        } else {
            current.push(column_name);
        }
        on_change.run(current);
        // Auto-collapse after adding from the list
        set_is_open.set(false);
    };

    // Remove column from selected list (no auto-collapse)
    let remove_column = move |column_name: String| {
        let mut current = selected_columns.get();
        current.retain(|c| c != &column_name);
        on_change.run(current);
    };

    // Move column up in the order
    let move_column_up = move |column_name: String| {
        let mut current = selected_columns.get();
        if let Some(idx) = current.iter().position(|c| c == &column_name) {
            if idx > 0 {
                current.swap(idx, idx - 1);
                on_change.run(current);
            }
        }
    };

    // Move column down in the order
    let move_column_down = move |column_name: String| {
        let mut current = selected_columns.get();
        if let Some(idx) = current.iter().position(|c| c == &column_name) {
            if idx < current.len() - 1 {
                current.swap(idx, idx + 1);
                on_change.run(current);
            }
        }
    };

    // Create a derived signal for the enumerated selected columns
    let selected_with_index = move || {
        selected_columns
            .get()
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>()
    };

    view! {
        <div class="mb-6">
            <button
                class="w-full px-4 py-3 rounded-lg bg-slate-800/50 border border-slate-700 hover:bg-slate-800 hover:border-slate-600 transition-all flex items-center justify-between text-left"
                on:click=move |_| set_is_open.update(|open| *open = !*open)
            >
                <span class="text-gray-300 font-medium">
                    {move || if is_open.get() {
                        "▼ Hide Column Selector"
                    } else {
                        "▶ Select Columns"
                    }}
                </span>
                <span class="text-sm text-purple-400 font-mono">
                    {move || format!("{} selected", selected_columns.get().len())}
                </span>
            </button>

            <Show when=move || is_open.get()>
                <div class="mt-2 p-4 rounded-lg bg-slate-800/30 border border-slate-700 backdrop-blur-sm space-y-4">

                    // Selected columns with reorder controls
                    <div>
                        <div class="flex items-center justify-between mb-2">
                            <h3 class="text-sm font-semibold text-gray-300">"Selected Columns (drag to reorder)"</h3>
                            <button
                                class="px-2 py-1 text-xs rounded-md bg-slate-700 hover:bg-slate-600 text-gray-300 transition-colors"
                                on:click=on_deselect_all
                            >
                                "Clear All"
                            </button>
                        </div>
                        <div class="rounded-lg border border-slate-600 bg-slate-900/30 overflow-hidden">
                            <Show
                                when=move || !selected_columns.get().is_empty()
                                fallback=|| view! {
                                    <div class="px-4 py-8 text-center text-gray-500 text-sm">
                                        "No columns selected. Select from the list below."
                                    </div>
                                }
                            >
                                <For
                                    each=selected_with_index
                                    key=|(idx, name)| format!("{}-{}", idx, name)
                                    children=move |(idx, name)| {
                                        let name_for_up = name.clone();
                                        let name_for_down = name.clone();
                                        let name_for_remove = name.clone();
                                        let total = selected_columns.get().len();
                                        let is_first = idx == 0;
                                        let is_last = idx == total - 1;

                                        view! {
                                            <div class="flex items-center gap-2 px-3 py-2 hover:bg-slate-800/50 border-b border-slate-700/50 last:border-b-0 transition-colors">
                                                <div class="flex flex-col">
                                                    <button
                                                        class="text-gray-500 hover:text-purple-400 disabled:opacity-30 disabled:cursor-not-allowed"
                                                        disabled=is_first
                                                        on:click=move |_| move_column_up(name_for_up.clone())
                                                        title="Move up"
                                                    >
                                                        "▲"
                                                    </button>
                                                    <button
                                                        class="text-gray-500 hover:text-purple-400 disabled:opacity-30 disabled:cursor-not-allowed"
                                                        disabled=is_last
                                                        on:click=move |_| move_column_down(name_for_down.clone())
                                                        title="Move down"
                                                    >
                                                        "▼"
                                                    </button>
                                                </div>
                                                <span class="flex-1 font-mono text-sm text-purple-300">
                                                    {name.clone()}
                                                </span>
                                                <button
                                                    class="text-red-400 hover:text-red-300 text-sm"
                                                    on:click=move |_| remove_column(name_for_remove.clone())
                                                    title="Remove column"
                                                >
                                                    "✕"
                                                </button>
                                            </div>
                                        }
                                    }
                                />
                            </Show>
                        </div>
                    </div>

                    // Divider
                    <hr class="border-slate-700" />

                    // All available columns
                    <div>
                        <h3 class="text-sm font-semibold text-gray-300 mb-2">"Add Columns"</h3>

                        // Search box
                        <input
                            type="text"
                            class="w-full px-4 py-2 mb-3 rounded-lg bg-slate-900/50 border border-slate-600 text-gray-200 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                            placeholder="🔍 Search columns by name or description..."
                            prop:value=move || search_term.get()
                            on:input=move |e: Event| {
                                set_search_term.set(event_target_value(&e));
                            }
                        />

                        // Action button
                        <div class="flex gap-2 mb-3">
                            <button
                                class="px-3 py-1.5 text-sm rounded-md bg-purple-600 hover:bg-purple-700 text-white transition-colors"
                                on:click=on_select_all
                            >
                                "Select All"
                            </button>
                        </div>

                        // Column list with scroll
                        <div class="max-h-80 overflow-y-auto rounded-lg border border-slate-600 bg-slate-900/30">
                            <For
                                each=filtered_columns
                                key=|(name, _)| name.clone()
                                children=move |(name, meta)| {
                                    let name_for_check = name.clone();
                                    let name_for_toggle = name.clone();
                                    let name_for_display = name.clone();
                                    let is_checked = move || selected_columns.get().contains(&name_for_check);

                                    view! {
                                        <label class="flex items-start gap-3 px-4 py-3 hover:bg-slate-800/50 cursor-pointer border-b border-slate-700/50 last:border-b-0 transition-colors">
                                            <input
                                                type="checkbox"
                                                class="mt-1 w-4 h-4 rounded border-gray-600 text-purple-600 focus:ring-purple-500 focus:ring-offset-0 bg-slate-700 cursor-pointer"
                                                checked=is_checked
                                                on:change=move |_| add_column(name_for_toggle.clone())
                                            />
                                            <div class="flex-1 min-w-0">
                                                <div class="flex items-baseline gap-2 flex-wrap">
                                                    <span class="font-mono text-sm font-semibold text-purple-300">
                                                        {name_for_display}
                                                    </span>
                                                    {meta.unit.as_ref().map(|unit| {
                                                        view! {
                                                            <span class="text-xs text-gray-500 font-mono">
                                                                "["{unit.clone()}"]"
                                                            </span>
                                                        }
                                                    })}
                                                </div>
                                                {meta.description.as_ref().map(|desc| {
                                                    view! {
                                                        <p class="text-xs text-gray-400 mt-1 leading-relaxed">
                                                            {desc.clone()}
                                                        </p>
                                                    }
                                                })}
                                            </div>
                                        </label>
                                    }
                                }
                            />
                        </div>

                        // Show count of filtered results
                        <div class="mt-2 text-xs text-gray-500 text-center">
                            {move || {
                                let filtered = filtered_columns();
                                let total = available_columns.get().len();
                                if filtered.len() < total {
                                    format!("Showing {} of {} columns", filtered.len(), total)
                                } else {
                                    format!("{} columns available", total)
                                }
                            }}
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
