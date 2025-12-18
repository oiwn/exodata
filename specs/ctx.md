# Current Context

## Table Components Refactoring Plan

### Current Problems

**stellarhosts_table.rs (227 lines)** - Doing too much:
- ❌ URL query parameter parsing (lines 16-29)
- ❌ State management for pagination/sorting (lines 32-34)
- ❌ Resource fetching with hardcoded server function (lines 37-42)
- ❌ Pagination logic calculations (lines 45-60)
- ❌ Sorting logic with URL navigation (lines 63-99)
- ❌ Inline pagination controls UI (lines 153-203)
- ❌ Page-specific layout/header (lines 102-117)
- ❌ Hardcoded route `/stellarhosts` in 3 places
- ❌ Hardcoded page size `50`

^^^ some looks weird but let's skip it for now

**table.rs** - Not reusable:
- ❌ Hardcoded column formatters for stellar hosts (lines 113-122)
- ❌ `format_column_name` called directly (line 19)

### Proposed Component Architecture

```
┌─────────────────────────────────────────────┐
│  StellarHostsTablePage                      │  ← Page-specific (80 lines)
│  - Page layout, header, styling             │
│  - URL param parsing → initial state        │
│  - Creates signals & Resource               │
│  - Defines server function call             │
│  - Column selection & formatters            │
└──────────────────┬──────────────────────────┘
                   │ passes: resource, signals, config
         ┌─────────▼──────────┐
         │  PaginatedTable    │  ← Reusable container (120 lines)
         │  - Sorting handler │
         │  - URL navigation  │
         │  - Pagination calc │
         │  - Suspense/errors │
         └─────┬────────┬─────┘
               │        │
      ┌────────▼──┐  ┌─▼──────────────┐
      │  Table    │  │ PaginationCtrl │  ← Reusable UI (50 lines each)
      │ (w/props) │  │ - Prev/Next    │
      └───────────┘  │ - Page info    │
                     └────────────────┘
```

### Detailed Component Structure

#### **Table Component** (`src/components/table.rs`)
**Purpose**: Pure presentational component that renders table HTML

**Props**:
```rust
#[component]
pub fn Table(
    data: TableData,                                    // The data to display
    on_sort: Callback<String>,                          // Called when column header clicked
    current_sort_column: Option<String>,                // Which column is currently sorted
    current_sort_order: String,                         // "asc" or "desc"

    #[prop(optional)]
    columns: Option<Vec<String>>,                       // Which columns to show (None = all)

    #[prop(optional)]
    column_formatter: Option<Callback<String, String>>, // Format column header names

    #[prop(optional)]
    cell_formatter: Option<Callback<(String, Value), String>>, // Format cell values
) -> impl IntoView
```

**Responsibilities**:
- Render `<table>` with headers and rows
- **Filter columns**: If `columns` prop provided, only display those columns (important: dataset has 100+ columns, we show 5 by default)
- Apply formatters to headers and cells
- Handle sort indicator display (↑/↓)
- Emit sort events on header clicks
- Show empty state when no data

**Column Filtering Logic**:
```rust
// Inside component
let display_columns = if let Some(cols) = columns {
    // Only show specified columns
    cols.iter().filter(|col| data.columns.contains(col)).cloned().collect()
} else {
    // Show all columns from data
    data.columns.clone()
};

// Then use display_columns for rendering headers and cells
```

---

#### **PaginationControls Component** (`src/components/pagination.rs`)
**Purpose**: Pure presentational component for pagination UI

**Props**:
```rust
#[component]
pub fn PaginationControls(
    current_page: usize,
    total_pages: usize,
    total_records: usize,
    start_record: usize,
    end_record: usize,
    on_previous: Callback<()>,
    on_next: Callback<()>,
    can_go_prev: bool,
    can_go_next: bool,
) -> impl IntoView
```

**Responsibilities**:
- Render "Showing X-Y of Z records"
- Render Previous/Next buttons
- Handle button disabled states
- Emit events on button clicks

---

#### **PaginatedTable Component** (`src/components/paginated_table.rs`)
**Purpose**: Smart container that manages table state and coordinates sub-components

**Props**:
```rust
#[component]
pub fn PaginatedTable(
    resource: Resource<(usize, Option<String>, String), Result<TableData, ServerFnError>>,
    current_page: (ReadSignal<usize>, WriteSignal<usize>),
    sort_column: (ReadSignal<Option<String>>, WriteSignal<Option<String>>),
    sort_order: (ReadSignal<String>, WriteSignal<String>),
    base_route: String,

    #[prop(optional)]
    columns: Option<Vec<String>>,

    #[prop(optional)]
    column_formatter: Option<Callback<String, String>>,

    #[prop(optional)]
    cell_formatter: Option<Callback<(String, Value), String>>,

    #[prop(optional)]
    loading_message: Option<String>,
) -> impl IntoView
```

**Responsibilities**:
- Handle sort clicks and toggle logic (asc → desc → none)
- Update URL with query parameters when state changes
- Calculate pagination metadata (total_pages, can_go_prev/next, start/end records)
- Render `<Suspense>` with loading spinner
- Render error states
- Pass data to `Table` component
- Pass pagination state to `PaginationControls` component

**Key Logic**:
```rust
// Sorting handler
let on_sort = Callback::new({
    let (_, set_sort_column) = sort_column;
    let (_, set_sort_order) = sort_order;
    let (_, set_current_page) = current_page;
    let navigate = use_navigate();

    move |column: String| {
        // Toggle sort: asc → desc → none
        // Reset to page 1
        // Update URL with build_table_query()
        // navigate(url)
    }
});

// Pagination calculations
let total_pages = move || {
    resource.get()
        .and_then(|res| res.ok())
        .map(|data| (data.total + data.limit - 1) / data.limit)
        .unwrap_or(1)
};

let can_go_prev = move || current_page.0.get() > 1;
let can_go_next = move || current_page.0.get() < total_pages();
```

**Render Structure**:
```rust
view! {
    <Suspense fallback=loading_spinner>
        {move || {
            resource.get().map(|result| match result {
                Ok(data) => view! {
                    <div class="space-y-6">
                        <Table
                            data=data
                            on_sort=on_sort
                            columns=columns
                            column_formatter=column_formatter
                            // ...
                        />
                        <PaginationControls
                            current_page=current_page.0.get()
                            total_pages=total_pages()
                            on_previous=on_prev_click
                            on_next=on_next_click
                            // ...
                        />
                    </div>
                },
                Err(e) => view! { <ErrorDisplay error=e /> }
            })
        }}
    </Suspense>
}
```

---

### Step-by-Step Refactoring

#### **Step 1: Extract PaginationControls Component**
**File**: `src/components/pagination.rs` (new)

**What to extract**: Lines 153-203 from stellarhosts_table.rs

**Props**:
```rust
- current_page: usize
- total_pages: usize
- total_records: usize
- start_record: usize
- end_record: usize
- on_previous: Callback<()>
- on_next: Callback<()>
- can_go_prev: bool
- can_go_next: bool
```

**Result**: Pure presentational component, no business logic

---

#### **Step 2: Make Table Accept Column Formatter Prop**
**File**: `src/components/table.rs`

**Changes**:
- Remove hardcoded `format_column_name` (lines 113-122)
- Add optional prop: `column_formatter: Option<Callback<String, String>>`
- Use formatter if provided, otherwise use column name as-is
- Keep `format_cell_value` as default (can be made configurable later)

**Result**: Table becomes reusable for any data type

---

#### **Step 3: Create PaginatedTable Component**
**File**: `src/components/paginated_table.rs` (new)

**What to extract**: Lines 11-99 + pagination rendering from stellarhosts_table.rs

**Props**:
```rust
#[component]
pub fn PaginatedTable(
    // Data - Resource created by parent page with its server function
    resource: Resource<(usize, Option<String>, String), Result<TableData, ServerFnError>>,

    // State management - signals from parent
    current_page: (ReadSignal<usize>, WriteSignal<usize>),
    sort_column: (ReadSignal<Option<String>>, WriteSignal<Option<String>>),
    sort_order: (ReadSignal<String>, WriteSignal<String>),

    // Configuration
    base_route: String,          // e.g., "/stellarhosts"

    // Column selection
    #[prop(optional)]
    columns: Option<Vec<String>>,  // Which columns to display (if None, show all)

    // Customization
    #[prop(optional)]
    column_formatter: Option<Callback<String, String>>,

    #[prop(optional)]
    cell_formatter: Option<Callback<(String, Value), String>>,  // (column_name, value) -> formatted_string

    // Loading/error customization
    #[prop(optional)]
    loading_message: Option<String>,
) -> impl IntoView
```

**Contains**:
- Sorting handler with URL navigation
- Pagination calculations (total_pages, can_go_prev/next)
- Navigation handlers (prev/next buttons)
- Suspense with loading spinner
- Error display
- Renders Table + PaginationControls

**Result**: Fully reusable paginated table logic

**Note**: Parent page creates the Resource with its server function, avoiding complex generic parameters

---

#### **Step 4: Simplify StellarHostsTablePage**
**File**: `src/components/stellarhosts_table.rs`

**New structure** (70-80 lines):
```rust
#[component]
pub fn StellarHostsTablePage() -> impl IntoView {
    let query_map = use_query_map();
    let navigate = use_navigate();

    // Initialize state from URL params (or defaults)
    let initial_page = query_map.with_untracked(|q| {
        q.get("page").and_then(|p| p.parse::<usize>().ok()).unwrap_or(1)
    });
    let initial_sort_column = query_map.with_untracked(|q| q.get("sort").map(|s| s.to_string()));
    let initial_sort_order = query_map.with_untracked(|q| {
        q.get("order").map(|o| o.to_string()).unwrap_or_else(|| "asc".to_string())
    });

    // Reactive state for pagination and sorting
    let (current_page, set_current_page) = signal(initial_page);
    let (sort_column, set_sort_column) = signal(initial_sort_column);
    let (sort_order, set_sort_order) = signal(initial_sort_order);

    // Resource that fetches data when dependencies change
    let table_resource = Resource::new(
        move || (current_page.get(), sort_column.get(), sort_order.get()),
        move |(page, sort_col, order)| async move {
            get_stellarhosts_page(page, 50, sort_col, Some(order)).await
        },
    );

    // Define which columns to display (out of hundreds available)
    let columns = vec![
        "hostname".to_string(),
        "sy_dist".to_string(),
        "st_teff".to_string(),
        "st_mass".to_string(),
        "sy_pnum".to_string(),
    ];

    // Define stellar hosts specific column formatter
    let column_formatter = Callback::new(|col: String| {
        match col.as_str() {
            "hostname" => "Star Name".to_string(),
            "sy_dist" => "Distance (pc)".to_string(),
            "st_teff" => "Temperature (K)".to_string(),
            "st_mass" => "Mass (M☉)".to_string(),
            "sy_pnum" => "Planets".to_string(),
            _ => col,
        }
    });

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
            <div class="container mx-auto px-4 py-8">
                <div class="mb-8">
                    <A href="/">"← Back to Overview"</A>
                    <h1>"⭐ Stellar Hosts Catalog"</h1>
                    <p>"Browse the complete database of confirmed stellar host systems"</p>
                </div>

                <PaginatedTable
                    resource=table_resource
                    current_page=(current_page, set_current_page)
                    sort_column=(sort_column, set_sort_column)
                    sort_order=(sort_order, set_sort_order)
                    base_route="/stellarhosts".to_string()
                    columns=Some(columns)
                    column_formatter=Some(column_formatter)
                />
            </div>
        </div>
    }
}
```

**Result**: Clean, focused page component. Page owns the Resource and server function call. Easy to create new table pages.

---

### Benefits

1. ✅ **Reusability**: Creating a new paginated table = 50 lines instead of 227
2. ✅ **Separation of concerns**: Page layout ≠ table logic ≠ UI controls
3. ✅ **Testability**: Each component can be tested independently
4. ✅ **Maintainability**: Changes to pagination logic happen in one place
5. ✅ **Flexibility**: Easy to customize per page (formatters, page size, styling)
6. ✅ **DRY**: No code duplication when adding exoplanets table, missions table, etc.

---

### Migration Order

1. **Start**: PaginationControls (simplest, pure UI)
2. **Then**: Make Table accept formatter prop (small change)
3. **Then**: Create PaginatedTable (complex, but builds on previous steps)
4. **Finally**: Refactor StellarHostsTablePage to use new components

This order minimizes risk - each step can be tested independently.

---

### Future Enhancements (Optional)
- Make cell formatter customizable (similar to column formatter)
- Add search/filter support to PaginatedTable
- Support custom loading spinners
- Add keyboard navigation (arrow keys for pagination)

