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
- Table component should be dumb renderer, we should be able to have all parameters for render in get request.

### Data Structure

```rust
pub struct TableData {
    pub columns: Vec<String>,                    // All column names from parquet
    pub rows: Vec<HashMap<String, Value>>,       // Generic row data
    pub total: usize,                            // Filtered count (for pagination)
    pub total_all: usize,                        // Unfiltered count (entire dataset)
    pub page: usize,
    pub limit: usize,
}
```

**Why this works**:
- HashMap<String, Value> is completely generic. No specific structs needed!
- Two totals give full context: "Showing 10 of 500 records (2000 total in dataset)"

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

### Backend Filtering Logic (Sorting with Nulls)

**Problem**: When sorting by a column, many rows may have null values for that column (since datasets have 100+ optional columns).

**Solution**: Filter out null rows at the **Polars query level** using predicates, not manual iteration.

**Implementation** (`src/server/common.rs`):

```rust
pub fn get_stellarhosts_data(
    df: &DataFrame,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
) -> Result<(Vec<Value>, usize, usize, Vec<String>), String> {
    let mut df = df.clone();

    // Select columns
    df = df.select(columns_to_select)?;

    // Get unfiltered total FIRST
    let total_all = df.height();

    // Apply sorting with null filtering
    if let Some(sort_col) = &sort_by {
        // Filter out rows where sort column is null (Polars predicate)
        df = df.filter(&col(sort_col).is_not_null())
            .map_err(|e| format!("Failed to filter nulls: {}", e))?;

        // Then sort
        let descending = order.as_deref().unwrap_or("asc") == "desc";
        let options = SortMultipleOptions::new().with_order_descending(descending);
        df = df.sort([sort_col.as_str()], options)
            .map_err(|e| format!("Failed to sort: {}", e))?;
    }

    // Get filtered total AFTER filtering
    let total = df.height();

    // Apply pagination...

    Ok((rows, total, total_all, columns))
}
```

**Why this approach**:
- ✅ **Performance**: Polars can skip entire row groups where column is null
- ✅ **Efficiency**: Don't read rows we'll discard
- ✅ **Scalability**: Works with millions of rows
- ✅ **Two totals**: UI can show "500 of 2000 total" context

---

### Implementation Plan

#### **Step 0: Update Backend Data Structures**

**Files**: `src/server/functions.rs` and `src/server/common.rs`

**Changes**:
1. Update `TableData` struct to include `total_all: usize`
2. Update `StellarHostsResult` type to return `(Vec<Value>, usize, usize, Vec<String>)`
3. In `get_stellarhosts_data()`:
   - Get `total_all` before filtering
   - Add null filtering when `sort_by` is specified: `df.filter(&col(sort_col).is_not_null())`
   - Get `total` after filtering
   - Return both totals
4. Update `get_stellarhosts_page()` to pass `total_all` to `TableData`

**Test**: Existing tests should still pass with updated signatures

---

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

