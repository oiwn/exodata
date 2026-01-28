use crate::server::functions::ColumnMetadata;
use leptos::ev::Event;
use leptos::prelude::*;
use std::collections::HashMap;

/// A single row in the selected columns list with reorder controls
#[component]
fn SelectedColumnItem(
    name: String,
    idx: usize,
    total: usize,
    on_move_up: Callback<String>,
    on_move_down: Callback<String>,
    on_remove: Callback<String>,
) -> impl IntoView {
    let name_for_up = name.clone();
    let name_for_down = name.clone();
    let name_for_remove = name.clone();
    let is_first = idx == 0;
    let is_last = idx == total - 1;

    view! {
        <div class="flex items-center gap-2 px-3 py-2 hover:bg-slate-800/50 border-b border-slate-700/50 last:border-b-0 transition-colors">
            <div class="flex flex-col">
                <button
                    class="text-gray-500 hover:text-purple-400 disabled:opacity-30 disabled:cursor-not-allowed"
                    disabled=is_first
                    on:click=move |_| on_move_up.run(name_for_up.clone())
                    title="Move up"
                >
                    "▲"
                </button>
                <button
                    class="text-gray-500 hover:text-purple-400 disabled:opacity-30 disabled:cursor-not-allowed"
                    disabled=is_last
                    on:click=move |_| on_move_down.run(name_for_down.clone())
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
                on:click=move |_| on_remove.run(name_for_remove.clone())
                title="Remove column"
            >
                "✕"
            </button>
        </div>
    }
}

/// A single row in the available columns list with checkbox
#[component]
fn AvailableColumnItem(
    name: String,
    meta: ColumnMetadata,
    selected_columns: ReadSignal<Vec<String>>,
    on_toggle: Callback<String>,
) -> impl IntoView {
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
                on:change=move |_| on_toggle.run(name_for_toggle.clone())
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

/// List of selected columns with reorder controls
#[component]
fn SelectedColumnsList(
    selected_columns: ReadSignal<Vec<String>>,
    on_move_up: Callback<String>,
    on_move_down: Callback<String>,
    on_remove: Callback<String>,
    on_clear_all: Callback<()>,
) -> impl IntoView {
    let selected_with_index = move || {
        let cols = selected_columns.get();
        let total = cols.len();
        cols.into_iter()
            .enumerate()
            .map(|(idx, name)| (idx, name, total))
            .collect::<Vec<_>>()
    };

    view! {
        <div>
            <div class="flex items-center justify-between mb-2">
                <h3 class="text-sm font-semibold text-gray-300">"Selected Columns (drag to reorder)"</h3>
                <button
                    class="px-2 py-1 text-xs rounded-md bg-slate-700 hover:bg-slate-600 text-gray-300 transition-colors"
                    on:click=move |_| on_clear_all.run(())
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
                        key=|(idx, name, _)| format!("{}-{}", idx, name)
                        children=move |(idx, name, total)| {
                            view! {
                                <SelectedColumnItem
                                    name=name
                                    idx=idx
                                    total=total
                                    on_move_up=on_move_up
                                    on_move_down=on_move_down
                                    on_remove=on_remove
                                />
                            }.into_any()
                        }
                    />
                </Show>
            </div>
        </div>
    }
}

/// List of available columns with search and checkboxes
#[component]
fn AvailableColumnsList(
    available_columns: ReadSignal<HashMap<String, ColumnMetadata>>,
    selected_columns: ReadSignal<Vec<String>>,
    on_toggle: Callback<String>,
    on_select_all: Callback<()>,
) -> impl IntoView {
    let (search_term, set_search_term) = signal(String::new());

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

    view! {
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
                    on:click=move |_| on_select_all.run(())
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
                        view! {
                            <AvailableColumnItem
                                name=name
                                meta=meta
                                selected_columns=selected_columns
                                on_toggle=on_toggle
                            />
                        }.into_any()
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
    }
}

#[component]
pub fn ColumnSelector(
    /// All available columns with their metadata
    available_columns: ReadSignal<HashMap<String, ColumnMetadata>>,
    /// Currently selected column names
    selected_columns: ReadSignal<Vec<String>>,
    /// Callback when selection changes
    on_change: Callback<Vec<String>>,
    /// External control for open/close state
    #[prop(optional)]
    is_open: Option<ReadSignal<bool>>,
    /// Callback when open/close state changes
    #[prop(optional)]
    on_toggle: Option<Callback<bool>>,
) -> impl IntoView {
    // Use external is_open if provided, otherwise create internal state
    let (internal_is_open, set_internal_is_open) = signal(false);
    let is_open_signal = is_open.unwrap_or(internal_is_open);

    let toggle_open = move |new_state: bool| {
        if let Some(callback) = on_toggle {
            callback.run(new_state);
        } else {
            set_internal_is_open.set(new_state);
        }
    };

    // Select all handler
    let on_select_all = Callback::new(move |_: ()| {
        let all: Vec<String> = available_columns.get().keys().cloned().collect();
        on_change.run(all);
    });

    // Deselect all handler
    let on_clear_all = Callback::new(move |_: ()| {
        on_change.run(vec![]);
    });

    // Toggle column in selection
    let on_toggle_column = Callback::new(move |column_name: String| {
        let mut current = selected_columns.get();
        if current.contains(&column_name) {
            current.retain(|c| c != &column_name);
        } else {
            current.push(column_name);
        }
        on_change.run(current);
    });

    // Remove column from selection
    let on_remove = Callback::new(move |column_name: String| {
        let mut current = selected_columns.get();
        current.retain(|c| c != &column_name);
        on_change.run(current);
    });

    // Move column up
    let on_move_up = Callback::new(move |column_name: String| {
        let mut current = selected_columns.get();
        if let Some(idx) = current.iter().position(|c| c == &column_name) {
            if idx > 0 {
                current.swap(idx, idx - 1);
                on_change.run(current);
            }
        }
    });

    // Move column down
    let on_move_down = Callback::new(move |column_name: String| {
        let mut current = selected_columns.get();
        if let Some(idx) = current.iter().position(|c| c == &column_name) {
            if idx < current.len() - 1 {
                current.swap(idx, idx + 1);
                on_change.run(current);
            }
        }
    });

    view! {
        <div class="mb-6">
            <button
                class="w-full px-4 py-3 rounded-lg bg-slate-800/50 border border-slate-700 hover:bg-slate-800 hover:border-slate-600 transition-all flex items-center justify-between text-left"
                on:click=move |_| {
                    let new_state = !is_open_signal.get();
                    toggle_open(new_state);
                }
            >
                <span class="text-gray-300 font-medium">
                    {move || if is_open_signal.get() {
                        "▼ Hide Column Selector"
                    } else {
                        "▶ Select Columns"
                    }}
                </span>
                <span class="text-sm text-purple-400 font-mono">
                    {move || format!("{} selected", selected_columns.get().len())}
                </span>
            </button>

            <Show when=move || is_open_signal.get()>
                {view! {
                    <div class="mt-2 p-4 rounded-lg bg-slate-800/30 border border-slate-700 backdrop-blur-sm space-y-4">
                        <SelectedColumnsList
                            selected_columns=selected_columns
                            on_move_up=on_move_up
                            on_move_down=on_move_down
                            on_remove=on_remove
                            on_clear_all=on_clear_all
                        />

                        <hr class="border-slate-700" />

                        <AvailableColumnsList
                            available_columns=available_columns
                            selected_columns=selected_columns
                            on_toggle=on_toggle_column
                            on_select_all=on_select_all
                        />
                    </div>
                }.into_any()}
            </Show>
        </div>
    }
}
