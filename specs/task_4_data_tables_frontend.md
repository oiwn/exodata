## 4. Data Tables Frontend

This document outlines the implementation specification for building a simple, tabbed interface to display stellar hosts and exoplanets data in sortable tables.

### 4.1. Goal

Create a user-friendly interface with tabs for browsing stellar hosts and exoplanets data, with basic sorting and pagination, enabling users to explore the full datasets.

### 4.2. Scope

#### 4.2.1. What's Included
- Tabbed interface with two tabs: "Stellar Hosts" and "Exoplanets"
- Data tables displaying key columns for each dataset
- Client-side sorting (click column headers)
- Pagination controls (previous/next, page numbers)
- Loading states during data fetch
- Error handling and empty state messages
- Responsive layout (works on mobile and desktop)

#### 4.2.2. What's NOT Included
- ❌ Advanced filtering (text search, range filters)
- ❌ Column visibility toggles
- ❌ Row selection/bulk actions
- ❌ Data export functionality
- ❌ Detail pages (click-through to individual records)
- ❌ Bookmarking/favorites
- ❌ Visualizations or charts

### 4.3. User Interface Design

#### 4.3.1. Page Structure
```
┌─────────────────────────────────────────┐
│ Exoplanet Catalog                       │
│ [Home] [Data Tables]                    │
├─────────────────────────────────────────┤
│                                         │
│ ┌─ Stellar Hosts ─┐ ┌─ Exoplanets ─┐  │
│ │   (active)      │ │   (inactive) │   │
│ └─────────────────┘ └──────────────┘   │
│                                         │
│ ┌─────────────────────────────────────┐│
│ │ Hostname  | Distance | Temp | ...   ││
│ │ ─────────────────────────────────── ││
│ │ HD 12345  | 45.2 pc  | 5778 | ...   ││
│ │ HD 67890  | 120.5 pc | 6200 | ...   ││
│ │ ...                                  ││
│ └─────────────────────────────────────┘│
│                                         │
│ [Previous] Page 1 of 938 [Next]        │
│ Showing 50 of 46,887 records           │
└─────────────────────────────────────────┘
```

#### 4.3.2. Navigation
- Top navigation with "Home" (overview) and "Data Tables"
- Tabs switch datasets without page reload
- URL updates with tab state: `/data?tab=stellarhosts` or `/data?tab=exoplanets`

### 4.4. Data Table Specifications

#### 4.4.1. Stellar Hosts Table Columns
Display these columns (subset of 136 total):
- `hostname` - Star name/identifier
- `sy_dist` - Distance to system (parsecs)
- `st_teff` - Effective temperature (Kelvin)
- `st_mass` - Stellar mass (solar masses)
- `st_rad` - Stellar radius (solar radii)
- `sy_pnum` - Number of planets in system

#### 4.4.2. Exoplanets Table Columns
Display these columns (subset of 355 total):
- `pl_name` - Planet name
- `hostname` - Host star name
- `discoverymethod` - Discovery method
- `disc_year` - Discovery year
- `pl_orbper` - Orbital period (days)
- `pl_rade` - Planet radius (Earth radii)
- `pl_masse` - Planet mass (Earth masses)

#### 4.4.3. Table Features
- **Sorting**: Click column header to sort (ascending/descending)
- **Pagination**: 50 rows per page by default
- **Null Handling**: Display "—" for missing values
- **Number Formatting**: Round floats to 2 decimal places
- **Header Tooltips**: Show full column name on hover

### 4.5. Technical Implementation

#### 4.5.1. Frontend Components
```
src/components/
├── mod.rs              [UPDATE]
├── overview.rs         [EXISTS from Task 2]
├── data_tables.rs      [NEW] Main page component
├── table.rs            [NEW] Reusable table component
└── tabs.rs             [NEW] Tab navigation component
```

#### 4.5.2. Server Functions
```rust
// New server functions in src/server/functions.rs

#[server]
async fn get_stellarhosts_page(
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
) -> Result<TableData, ServerFnError>

#[server]
async fn get_exoplanets_page(
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
) -> Result<TableData, ServerFnError>
```

#### 4.5.3. Data Structures
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct TableData {
    pub rows: Vec<HashMap<String, Value>>,
    pub total: usize,
    pub page: usize,
    pub limit: usize,
    pub columns: Vec<String>,
}
```

#### 4.5.4. Routing
Add to `src/app.rs`:
```rust
<Route path=StaticSegment("data") view=DataTablesPage/>
```

### 4.6. User Interactions

#### 4.6.1. Tab Switching
- Click "Exoplanets" tab → loads first page of exoplanets data
- Click "Stellar Hosts" tab → loads first page of stellar hosts data
- Active tab highlighted with different background color
- Tab state preserved in URL query parameter

#### 4.6.2. Sorting
- Click column header → sort ascending
- Click again → sort descending
- Click again → remove sort (default order)
- Visual indicator shows sort direction (↑/↓ arrow)

#### 4.6.3. Pagination
- "Previous" button disabled on first page
- "Next" button disabled on last page
- Show current page and total pages
- Show record range (e.g., "Showing 51-100 of 46,887")

### 4.7. State Management

#### 4.7.1. URL State
Encode table state in URL:
- `?tab=exoplanets` - Active tab
- `&page=5` - Current page
- `&sort=disc_year` - Sort column
- `&order=desc` - Sort direction

#### 4.7.2. Component State
Use Leptos signals for:
- `active_tab` - Which dataset is displayed
- `current_page` - Page number
- `sort_column` - Active sort column
- `sort_order` - Sort direction (asc/desc)

### 4.8. Success Criteria

#### 4.8.1. Functional Requirements
- ✅ Both tabs load data correctly
- ✅ Pagination works (previous/next/page numbers)
- ✅ Sorting works on all displayed columns
- ✅ Tab switches without full page reload
- ✅ Loading spinner shown during data fetch
- ✅ Empty state message if no data
- ✅ Error message if fetch fails
- ✅ URL updates with state changes
- ✅ Browser back/forward buttons work

#### 4.8.2. Performance Requirements
- ✅ Initial page load < 300ms
- ✅ Tab switch < 200ms
- ✅ Pagination < 150ms
- ✅ Sorting < 150ms (may increase for server-side sort)

#### 4.8.3. UX Requirements
- ✅ Responsive on mobile (table scrolls horizontally)
- ✅ Clear visual feedback for interactive elements
- ✅ Consistent styling with overview page
- ✅ Accessible (keyboard navigation works)

### 4.9. Styling Guidelines

- Use Tailwind CSS for consistency
- Table: bordered rows, hover effect on rows
- Headers: bold, clickable, sort indicator
- Pagination: centered, clear buttons
- Tabs: clear active/inactive states
- Mobile: horizontal scroll for table, stacked pagination buttons

### 4.10. Error Handling

- **No data**: Show "No records found" message
- **Fetch error**: Show error banner with retry button
- **Invalid page**: Redirect to page 1
- **Invalid sort column**: Ignore and use default

### 4.11. Future Enhancements (Not in Task 4)

- Search/filter functionality (Task 5+)
- Column visibility toggles
- Adjustable page size (10/50/100/500)
- Detail page links (click row to view full record)
- Data export (CSV, JSON)
- Advanced filtering UI
- Saved filter presets

### 4.12. File Structure After Task 4

```
src/components/
├── mod.rs              [UPDATE] Export new components
├── overview.rs         [EXISTS]
├── data_tables.rs      [NEW] Main page with tabs
├── table.rs            [NEW] Reusable table component
└── tabs.rs             [NEW] Tab navigation component

src/server/
└── functions.rs        [UPDATE] Add pagination server functions

src/app.rs              [UPDATE] Add /data route
```

### 4.13. Testing Approach

- Visual testing in browser (desktop and mobile)
- Test tab switching preserves no state bleed
- Test pagination edge cases (first/last page)
- Test sorting (asc, desc, clear)
- Test URL state persistence (refresh page, back button)
- Test with slow network (loading states)
- Test error scenarios (disconnect server)

### 4.14. Dependencies

This task depends on:
- ✅ Task 1: Parquet data available
- ✅ Task 2: Catalog module and server functions infrastructure

This task enables:
- 🔲 Task 5: Search and filtering (extends table component)
- 🔲 Task 6: Detail pages (links from table rows)
- 🔲 Future: Data export functionality

### 4.15. Implementation Strategy

Implement iteratively:

1. **Basic Structure**: Create data_tables.rs with tabs UI (no data)
2. **Table Component**: Build reusable table with hardcoded data
3. **Server Functions**: Add get_*_page functions
4. **Data Integration**: Connect server functions to table component
5. **Pagination**: Add pagination controls and logic
6. **Sorting**: Add column sorting functionality
7. **URL State**: Sync state with URL parameters
8. **Polish**: Loading states, error handling, styling
