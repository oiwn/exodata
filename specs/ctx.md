# Current Context

## Generic Table Component - Simplified Plan

### Core Philosophy

**Keep It Simple**: One generic table component that renders ANY columnar data. Pages handle ALL the logic.

### The Problem We're Solving

- Data comes from Parquet files (columnar format) with **100+ columns**
- Each table type (stellar hosts, exoplanets, etc.) has different columns
- We need to show only 5-10 columns at a time (not all 100+)
- Current code has hardcoded column names and formatters
- We want ONE generic table that works for everything

### Data Structure (Already Perfect!)

```rust
// This already exists and is exactly what we need
pub struct TableData {
    pub columns: Vec<String>,                    // All column names from parquet
    pub rows: Vec<HashMap<String, Value>>,       // Generic row data
    pub total: usize,                            // Total records (for pagination)
    pub page: usize,
    pub limit: usize,
}
```

**Why this works**: HashMap<String, Value> is completely generic. No specific structs needed!

### Proposed Architecture

```
┌─────────────────────────────────────┐
│  Page Component                     │ ← Page does ALL the work
│  (e.g., StellarHostsTablePage)      │
│                                     │
│  - Parse URL params                 │
│  - Create signals (page, sort)      │
│  - Create Resource with server fn   │
│  - Define visible_columns (5 of 100)│
│  - Define column display names      │
│  - Handle sort clicks               │
│  - Handle pagination clicks         │
│  - Update URL on changes            │
│  - Suspense/error handling          │
└────────────┬────────────────────────┘
             │ passes: TableData + config
             ▼
┌─────────────────────────────────────┐
│  GenericTable                       │ ← Dumb HTML renderer
│                                     │
│  - Render <table> HTML              │
│  - Loop through visible_columns     │
│  - Show display names               │
│  - Show sort indicators (↑/↓)       │
│  - Emit column clicks               │
│  - That's it!                       │
└─────────────────────────────────────┘
```

**Key principle**: GenericTable is stateless. It just renders what you tell it to render.

### Component Details

#### **GenericTable Component** (`src/components/table.rs`)

**Purpose**: Stateless HTML renderer. No logic, no state, just renders what you give it.

**Props**:
```rust
#[component]
pub fn GenericTable(
    // The data to render
    data: TableData,

    // Which columns to show (e.g., 5 out of 100)
    visible_columns: Vec<String>,

    // Display names for column headers
    // Example: "hostname" -> "Star Name", "sy_dist" -> "Distance (pc)"
    column_display_names: HashMap<String, String>,

    // Optional: sorting state
    #[prop(optional)]
    sorted_column: Option<String>,

    #[prop(optional)]
    sort_direction: Option<String>,  // "asc" or "desc"

    // Optional: emit column name when header clicked
    #[prop(optional)]
    on_column_click: Option<Callback<String>>,
) -> impl IntoView
```

**What it does**:
```rust
view! {
    <div class="overflow-x-auto rounded-xl border border-slate-700 bg-slate-800/50">
        <table class="w-full border-collapse">
            <thead>
                <tr>
                    {visible_columns.iter().map(|col| {
                        let display_name = column_display_names.get(col)
                            .unwrap_or(col);  // Fallback to column ID if no display name

                        let is_sorted = sorted_column.as_ref() == Some(col);
                        let sort_indicator = if is_sorted {
                            match sort_direction.as_deref() {
                                Some("asc") => " ↑",
                                Some("desc") => " ↓",
                                _ => "",
                            }
                        } else {
                            ""
                        };

                        // Render <th> with click handler if provided
                    }).collect::<Vec<_>>()}
                </tr>
            </thead>
            <tbody>
                {data.rows.iter().map(|row| {
                    view! {
                        <tr>
                            {visible_columns.iter().map(|col| {
                                let value = row.get(col).unwrap_or(&Value::Null);
                                let formatted = format_cell_value(value);
                                view! { <td>{formatted}</td> }
                            }).collect::<Vec<_>>()}
                        </tr>
                    }
                }).collect::<Vec<_>>()}
            </tbody>
        </table>

        {if data.rows.is_empty() {
            view! { <div>"No data available"</div> }.into_any()
        } else {
            view! { <div></div> }.into_any()
        }}
    </div>
}
```

**Helper function** (keep in same file):
```rust
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
```

**That's it!** The component is ~80 lines total.

---

### Implementation Plan

#### **Step 1: Refactor GenericTable**

**File**: `src/components/table.rs`

**Changes**:
1. Add `visible_columns: Vec<String>` prop
2. Add `column_display_names: HashMap<String, String>` prop
3. Remove hardcoded `format_column_name()` function
4. Loop only through `visible_columns` instead of `data.columns`
5. Look up display names from the HashMap
6. Keep existing styling and sort indicators

**Test**: Should still render stellarhosts table correctly (we'll update the page component next)

---

#### **Step 2: Update StellarHostsTablePage**

**File**: `src/components/stellarhosts_table.rs`

**Changes**:
1. Add visible columns definition:
   ```rust
   let visible_columns = vec![
       "hostname".to_string(),
       "sy_dist".to_string(),
       "st_teff".to_string(),
       "st_mass".to_string(),
       "sy_pnum".to_string(),
   ];
   ```

2. Add column display names HashMap:
   ```rust
   let mut column_names = HashMap::new();
   column_names.insert("hostname".to_string(), "Star Name".to_string());
   column_names.insert("sy_dist".to_string(), "Distance (pc)".to_string());
   column_names.insert("st_teff".to_string(), "Temperature (K)".to_string());
   column_names.insert("st_mass".to_string(), "Mass (M☉)".to_string());
   column_names.insert("sy_pnum".to_string(), "Planets".to_string());
   ```

3. Update Table component call:
   ```rust
   <GenericTable
       data=data
       visible_columns=visible_columns.clone()
       column_display_names=column_names.clone()
       sorted_column=sort_column.get()
       sort_direction=Some(sort_order.get())
       on_column_click=Some(on_sort)
   />
   ```

**Test**: Should work exactly like before, but now using the generic approach

---

#### **Step 3: (Optional) Extract PaginationControls**

If we notice pagination UI is getting duplicated across pages, extract it later.

For now: Keep pagination HTML inline in the page components.

---

### Benefits

1. ✅ **Truly generic** - works with ANY columnar data (HashMap<String, Value>)
2. ✅ **No type-specific structs** - handles 100+ columns gracefully
3. ✅ **Simple & clear** - GenericTable is ~80 lines, easy to understand
4. ✅ **Page controls everything** - easy to customize per table type
5. ✅ **Minimal refactoring** - only 2 files to change

### Future Tables

Adding a new table (e.g., exoplanets) becomes easy:

```rust
#[component]
pub fn ExoplanetsTablePage() -> impl IntoView {
    // Same pattern as stellarhosts:
    // 1. Create Resource with get_exoplanets_page()
    // 2. Define visible_columns
    // 3. Define column_display_names
    // 4. Render GenericTable
}
```

**~100 lines total** per new table page!

