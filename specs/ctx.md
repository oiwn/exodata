# Current Context

## Implementation Plan: Column Selector Widget

**Goal**: Add a column selector widget that allows users to choose which columns to display in the table, with state synchronized to URL parameters.

### User Experience

Users should be able to:
1. See a list of all available columns with checkboxes
2. Select/deselect columns to show/hide them in the table
3. Have their selection reflected in the URL (e.g., `?columns=pl_name,hostname,pl_orbper`)
4. Share URLs with specific column selections
5. See column descriptions/units in the selector (from metadata)

### Current State

✅ **Available:**
- Metadata for all columns (name, description, unit, datatype)
- `TableData` includes metadata in API response
- Server functions can accept query parameters

❌ **Missing:**
- Column selector UI component
- URL parameter handling for column selection
- Server-side column filtering based on query parameter
- State synchronization between UI, URL, and table

---

## Implementation Steps

### Step 1: Server-Side Column Filtering

**Files to modify:**
- `src/server/common.rs` - Add `columns` parameter to data functions
- `src/server/functions.rs` - Pass `columns` parameter through server functions

**Changes:**

```rust
// src/server/functions.rs
#[server(input = GetUrl)]
pub async fn get_exoplanets_page(
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    columns: Option<String>,  // NEW: Comma-separated column names
) -> Result<TableData, ServerFnError> {
    let state = expect_context::<ApiState>();

    // Parse columns parameter
    let selected_columns = columns.map(|s| {
        s.split(',').map(|col| col.trim().to_string()).collect::<Vec<_>>()
    });

    let (rows, total, total_all, columns, metadata) = common::get_exoplanets_data(
        &state.exoplanets_df,
        &state.exoplanets_metadata,
        page,
        limit,
        sort_by,
        order,
        selected_columns,  // NEW
    )
    .map_err(|e: String| -> ServerFnError { ServerFnError::ServerError(e) })?;

    Ok(TableData {
        rows,
        columns,
        total,
        total_all,
        page,
        limit,
        metadata,
    })
}
```

```rust
// src/server/common.rs
pub fn get_exoplanets_data(
    df: &DataFrame,
    all_metadata: &Arc<HashMap<String, ColumnMetadata>>,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
    selected_columns: Option<Vec<String>>,  // NEW: Optional column filter
) -> ExoplanetsResult {
    let mut df = df.clone();

    // Define default columns if none specified
    let default_columns = vec![
        "pl_name",
        "hostname",
        "discoverymethod",
        "disc_year",
        "pl_orbper",
        "pl_rade",
        "pl_bmasse",
    ];

    // Use selected columns or fall back to defaults
    let columns_to_select: Vec<&str> = if let Some(cols) = &selected_columns {
        // Validate that requested columns exist in dataframe
        cols.iter()
            .filter(|col| df.column(col).is_ok())
            .map(|s| s.as_str())
            .collect()
    } else {
        default_columns
    };

    // Ensure we have at least one column
    if columns_to_select.is_empty() {
        return Err("No valid columns selected".to_string());
    }

    // Select only the requested columns
    df = df
        .select(columns_to_select.clone())
        .map_err(|e| format!("Failed to select columns: {}", e))?;

    // ... rest of existing logic (sorting, pagination, etc.)
}
```

**Same for `get_stellarhosts_data()`**

---

### Step 2: Frontend Column Selector Component

**File to create**: `src/components/column_selector.rs`

**Component Features:**
- Display list of all available columns
- Show checkboxes for selection
- Display column descriptions from metadata
- Group columns by category (optional)
- "Select All" / "Deselect All" buttons
- Search/filter column list

**Component Structure:**

```rust
use leptos::prelude::*;
use std::collections::HashMap;
use exo_core::metadata::ColumnMetadata;

#[component]
pub fn ColumnSelector(
    /// All available columns with their metadata
    available_columns: HashMap<String, ColumnMetadata>,
    /// Currently selected column names
    selected_columns: Signal<Vec<String>>,
    /// Callback when selection changes
    on_change: impl Fn(Vec<String>) + 'static,
) -> impl IntoView {
    let (search_term, set_search_term) = signal(String::new());

    // Sort columns alphabetically
    let sorted_columns = move || {
        let mut cols: Vec<_> = available_columns.iter().collect();
        cols.sort_by_key(|(name, _)| *name);
        cols
    };

    // Filter columns based on search
    let filtered_columns = move || {
        sorted_columns()
            .into_iter()
            .filter(|(name, meta)| {
                let search = search_term.get().to_lowercase();
                if search.is_empty() {
                    return true;
                }
                name.to_lowercase().contains(&search) ||
                meta.description.as_ref()
                    .map(|d| d.to_lowercase().contains(&search))
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="column-selector">
            <h3>"Select Columns"</h3>

            // Search box
            <input
                type="text"
                placeholder="Search columns..."
                value=search_term
                on:input=move |e| set_search_term.set(event_target_value(&e))
            />

            // Select All / Deselect All
            <div class="selector-actions">
                <button on:click=move |_| {
                    let all: Vec<String> = available_columns.keys().cloned().collect();
                    on_change(all);
                }>
                    "Select All"
                </button>
                <button on:click=move |_| on_change(vec![])>
                    "Deselect All"
                </button>
            </div>

            // Column list with checkboxes
            <div class="column-list">
                <For
                    each=filtered_columns
                    key=|(name, _)| name.to_string()
                    children=move |(name, meta)| {
                        let is_checked = move || selected_columns.get().contains(name);
                        let name_clone = name.to_string();

                        view! {
                            <label class="column-item">
                                <input
                                    type="checkbox"
                                    checked=is_checked
                                    on:change=move |_| {
                                        let mut current = selected_columns.get();
                                        if current.contains(&name_clone) {
                                            current.retain(|c| c != &name_clone);
                                        } else {
                                            current.push(name_clone.clone());
                                        }
                                        on_change(current);
                                    }
                                />
                                <span class="column-name">{name}</span>
                                {meta.description.as_ref().map(|desc| {
                                    view! {
                                        <span class="column-desc">{desc}</span>
                                    }
                                })}
                                {meta.unit.as_ref().map(|unit| {
                                    view! {
                                        <span class="column-unit">"["{unit}"]"</span>
                                    }
                                })}
                            </label>
                        }
                    }
                />
            </div>
        </div>
    }
}
```

---

### Step 3: URL Parameter Synchronization

**File to modify**: Table components (`src/components/exoplanets_table.rs`, `src/components/stellarhosts_table.rs`)

**Approach:**
- Use Leptos router's query parameter utilities
- Read initial state from URL on mount
- Update URL when selection changes
- Listen to URL changes to update UI

**Implementation:**

```rust
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component]
pub fn ExoplanetsTable() -> impl IntoView {
    // Read URL parameters
    let query_params = use_query_map();

    // Parse columns from URL
    let initial_columns = move || {
        query_params
            .read()
            .get("columns")
            .map(|s| s.split(',').map(|col| col.trim().to_string()).collect())
            .unwrap_or_else(|| vec![
                "pl_name".to_string(),
                "hostname".to_string(),
                "discoverymethod".to_string(),
                "disc_year".to_string(),
                "pl_orbper".to_string(),
                "pl_rade".to_string(),
                "pl_bmasse".to_string(),
            ])
    };

    let (selected_columns, set_selected_columns) = signal(initial_columns());

    // Update URL when columns change
    let update_url = move |columns: Vec<String>| {
        set_selected_columns.set(columns.clone());

        // Update query parameter
        let columns_str = columns.join(",");
        let navigate = leptos_router::hooks::use_navigate();

        // Preserve other query params while updating columns
        let current_search = window().location().search().unwrap_or_default();
        let mut params = current_search
            .trim_start_matches('?')
            .split('&')
            .filter(|p| !p.starts_with("columns="))
            .collect::<Vec<_>>();

        if !columns.is_empty() {
            params.push(&format!("columns={}", columns_str));
        }

        let new_search = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

        navigate(&new_search, Default::default());
    };

    // Fetch data with selected columns
    let columns_param = move || Some(selected_columns.get().join(","));

    let data = Resource::new(
        move || (page.get(), limit.get(), sort_by.get(), order.get(), columns_param()),
        |(page, limit, sort_by, order, columns)| async move {
            get_exoplanets_page(page, limit, sort_by, order, columns).await
        },
    );

    view! {
        <div class="exoplanets-page">
            <ColumnSelector
                available_columns=/* get from metadata */
                selected_columns=selected_columns
                on_change=update_url
            />

            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || {
                    data.get().map(|result| match result {
                        Ok(table_data) => view! {
                            <Table
                                data=table_data
                                on_sort=/* ... */
                            />
                        },
                        Err(e) => view! { <p>"Error: " {e.to_string()}</p> }
                    })
                }}
            </Suspense>
        </div>
    }
}
```

---

### Step 4: Styling

**File to create/modify**: `style/main.scss` or component-specific styles

```scss
.column-selector {
    background: var(--surface-color);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;

    h3 {
        margin-top: 0;
    }

    input[type="text"] {
        width: 100%;
        padding: 0.5rem;
        margin-bottom: 1rem;
        border: 1px solid var(--border-color);
        border-radius: 4px;
    }

    .selector-actions {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 1rem;

        button {
            padding: 0.5rem 1rem;
            background: var(--primary-color);
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;

            &:hover {
                background: var(--primary-hover-color);
            }
        }
    }

    .column-list {
        max-height: 400px;
        overflow-y: auto;
        border: 1px solid var(--border-color);
        border-radius: 4px;
        padding: 0.5rem;

        .column-item {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.5rem;
            cursor: pointer;
            border-radius: 4px;

            &:hover {
                background: var(--hover-color);
            }

            input[type="checkbox"] {
                cursor: pointer;
            }

            .column-name {
                font-weight: 600;
                font-family: monospace;
                flex-shrink: 0;
            }

            .column-desc {
                color: var(--text-secondary);
                font-size: 0.9rem;
                flex: 1;
            }

            .column-unit {
                color: var(--text-tertiary);
                font-size: 0.85rem;
                font-style: italic;
            }
        }
    }
}
```

---

### Step 5: Advanced Features (Optional)

**Column Grouping:**
- Group columns by category (planet properties, stellar properties, discovery info, etc.)
- Collapsible sections for each group

**Presets:**
- Save common column selections as presets
- "Basic", "Discovery", "Orbital", "Physical" presets
- User-defined custom presets (localStorage)

**Column Reordering:**
- Drag-and-drop to reorder columns
- Order reflected in URL and table

**Persistence:**
- Remember user's last selection in localStorage
- Auto-restore on next visit

---

## Testing Strategy

**Unit Tests:**
- Column selector component renders correctly
- Checkbox state updates properly
- Search filtering works

**Integration Tests:**
- URL parameter parsing works
- Server correctly filters columns
- Table updates when columns change
- URL updates when selection changes

**Manual Testing:**
- Select/deselect columns and verify table updates
- Copy URL and open in new tab - selection should persist
- Search for columns
- Test with empty selection (should show error or default columns)
- Test with invalid column names in URL

---

## File Changes Summary

**New Files:**
- `src/components/column_selector.rs`

**Modified Files:**
- `src/server/common.rs` - Add `columns` parameter to data functions
- `src/server/functions.rs` - Add `columns` parameter to server functions
- `src/components/exoplanets_table.rs` - Integrate column selector + URL sync
- `src/components/stellarhosts_table.rs` - Integrate column selector + URL sync
- `src/components/mod.rs` - Export `ColumnSelector` component
- `style/main.scss` - Add column selector styles

**Tests to Update:**
- `src/server/common.rs` tests - Add column filtering tests
- Component tests for column selector

---

## URL Parameter Format

**Examples:**

```
# Select specific columns
/exoplanets?columns=pl_name,hostname,pl_orbper,pl_rade

# With pagination
/exoplanets?page=2&limit=50&columns=pl_name,hostname,disc_year

# With sorting and columns
/exoplanets?sort_by=pl_orbper&order=desc&columns=pl_name,pl_orbper,pl_rade

# No columns specified (use defaults)
/exoplanets
```

**URL Encoding:**
- Column names should be URL-encoded if they contain special characters
- Comma-separated list for multiple columns
- Empty or missing = use default columns

---

## Implementation Order

1. ✅ **Phase 1**: Server-side column filtering
   - Update `common.rs` functions
   - Update server functions
   - Add tests

2. ✅ **Phase 2**: Basic column selector component
   - Create component with checkboxes
   - Display all columns from metadata
   - Handle selection state

3. ✅ **Phase 3**: URL synchronization
   - Read columns from URL parameters
   - Update URL when selection changes
   - Integrate with table components

4. ✅ **Phase 4**: Styling and UX
   - Add CSS styles
   - Add search functionality
   - Add select all/deselect all

5. ⏳ **Phase 5**: Advanced features (optional)
   - Column grouping
   - Presets
   - localStorage persistence
   - Drag-and-drop reordering

---

## Next Steps

Start with **Phase 1**: Server-side column filtering in `src/server/common.rs`.
