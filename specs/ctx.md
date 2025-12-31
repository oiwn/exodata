# Current Context

## Implementation Plan: Metadata Integration

### Architecture Overview

**Current Data Flow**:
```
Frontend Component (ExoplanetsTablePage)
    ↓ (Resource::new with server function)
Server Function (get_exoplanets_page)
    ↓ (calls common::get_exoplanets_data)
Business Logic (src/server/common.rs)
    ↓ (reads from Arc<DataFrame>)
TableData Response
    ↓ (serialized to frontend)
Table Component (renders with column_descriptions)
```

**Gap**: TableData currently doesn't include column descriptions from metadata.rs

### Implementation Steps

#### Step 1: Load Metadata at Server Startup
**File**: `src/main.rs` (server startup)

**What to do**:
- Load VOTable metadata into memory alongside DataFrames
- Store in `ApiState` for reuse across requests
- Parse both `data/exoplanets.vot` and `data/stellarhosts.vot`

**Changes needed**:

```rust
// Add to ApiState struct
pub struct ApiState {
    pub exoplanets_df: Arc<DataFrame>,
    pub stellarhosts_df: Arc<DataFrame>,
    // NEW: Add metadata storage
    pub exoplanets_metadata: Arc<HashMap<String, ColumnMetadata>>,
    pub stellarhosts_metadata: Arc<HashMap<String, ColumnMetadata>>,
}
```

**Implementation**:
- Use `exo_core::metadata::parse_votable_metadata()` to load metadata
- Convert `Vec<ColumnMetadata>` to `HashMap<String, ColumnMetadata>` for fast lookup
- Wrap in `Arc<>` for cheap cloning across requests

**Error handling**:
- If VOTable file missing, log warning and use empty HashMap
- Application should still work without metadata (graceful degradation)

---

#### Step 2: Extend TableData Structure
**File**: `src/server/functions.rs`

**What to do**:
- Add `column_descriptions` field to `TableData` struct
- Make it match the Table component's expected type

**Changes needed**:
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct TableData {
    pub rows: Vec<serde_json::Value>,
    pub columns: Vec<String>,
    pub total: usize,
    pub total_all: usize,
    pub page: usize,
    pub limit: usize,
    // NEW: Add column descriptions
    pub column_descriptions: Option<HashMap<String, String>>,
}
```

---

#### Step 3: Update Business Logic to Include Metadata
**File**: `src/server/common.rs`

**What to do**:
- Modify `get_exoplanets_data()` and `get_stellarhosts_data()` functions
- Accept metadata as a parameter
- Build `column_descriptions` HashMap from metadata for requested columns

**Function signature changes**:
```rust
// OLD
pub fn get_exoplanets_data(
    df: Arc<DataFrame>,
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
) -> Result<(Vec<serde_json::Value>, usize, usize, Vec<String>), String>

// NEW
pub fn get_exoplanets_data(
    df: Arc<DataFrame>,
    metadata: Arc<HashMap<String, ColumnMetadata>>,  // NEW PARAM
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
) -> Result<(Vec<serde_json::Value>, usize, usize, Vec<String>, Option<HashMap<String, String>>), String>
//                                                                                      ^^^^^^^^^^^^^^^^
//                                                                                      NEW RETURN VALUE
```

**Implementation logic**:
```rust
// After determining which columns to return
let columns = vec!["pl_name", "hostname", "pl_orbper", ...];

// Build column descriptions from metadata
let column_descriptions: HashMap<String, String> = columns
    .iter()
    .filter_map(|col_name| {
        metadata.get(*col_name).and_then(|meta| {
            meta.description.as_ref().map(|desc| {
                // Optionally append unit to description
                let full_desc = match &meta.unit {
                    Some(unit) if !unit.is_empty() => format!("{} (Unit: {})", desc, unit),
                    _ => desc.clone(),
                };
                (col_name.to_string(), full_desc)
            })
        })
    })
    .collect();

let column_descriptions = if column_descriptions.is_empty() {
    None
} else {
    Some(column_descriptions)
};
```

---

#### Step 4: Update Server Functions
**File**: `src/server/functions.rs`

**What to do**:
- Modify `get_exoplanets_page()` and `get_stellarhosts_page()` server functions
- Extract metadata from `ApiState` via `use_context()`
- Pass metadata to business logic functions
- Include in TableData response

**Changes**:
```rust
#[server(GetExoplanetsPage, "/api")]
pub async fn get_exoplanets_page(
    page: usize,
    limit: usize,
    sort_by: Option<String>,
    order: Option<String>,
) -> Result<TableData, ServerFnError> {
    use crate::server::common::get_exoplanets_data;
    use leptos::use_context;

    let state = use_context::<ApiState>()
        .ok_or_else(|| ServerFnError::ServerError("ApiState not found".to_string()))?;

    // NEW: Extract metadata from state
    let metadata = state.exoplanets_metadata.clone();

    let (rows, total, total_all, columns, column_descriptions) =
        get_exoplanets_data(
            state.exoplanets_df.clone(),
            metadata,  // NEW: Pass metadata
            page,
            limit,
            sort_by,
            order,
        )
        .map_err(|e| ServerFnError::ServerError(e))?;

    Ok(TableData {
        rows,
        columns,
        total,
        total_all,
        page,
        limit,
        column_descriptions,  // NEW: Include in response
    })
}
```

---

#### Step 5: Update Frontend Components
**Files**: `src/components/exoplanets_table.rs`, `src/components/stellarhosts_table.rs`

**What to do**:
- Pass `column_descriptions` from `TableData` to `Table` component
- No logic changes needed - just wire through the data

**Changes**:
```rust
// In ExoplanetsTablePage component
<Table
    data=data.clone()
    on_sort=on_sort
    column_descriptions=data.column_descriptions.clone()  // CHANGED: Use from data instead of None
/>
```

**Current state**: Components already have this structure, just passing `None` currently

---

#### Step 6: Update REST API (Optional)
**File**: `src/server/handlers.rs`

**What to do** (if REST API is still in use):
- Add metadata field to `ApiResponse` struct
- Update handlers to include metadata
- Follow same pattern as server functions

**Note**: Based on architecture analysis, Leptos server functions are the primary data access method. REST API updates are optional for consistency.

---

### Testing Plan

1. **Unit test metadata loading**:
   - Verify `parse_votable_metadata()` works with sample files
   - Test HashMap conversion logic

2. **Integration test server functions**:
   - Mock ApiState with sample metadata
   - Call `get_exoplanets_page()` and verify `column_descriptions` field
   - Test with missing metadata (graceful degradation)

3. **Manual testing**:
   - Run server: `cargo leptos watch`
   - Navigate to exoplanets table
   - Hover over column headers and verify tooltips appear
   - Verify tooltip text matches NASA descriptions
   - Check browser console for errors

4. **Performance testing**:
   - Measure server startup time with metadata loading
   - Verify no significant impact on page load time
   - Metadata should be cached in memory (Arc<HashMap>)

---

### Performance Considerations

**Memory usage**:
- VOTable files are large (~400MB), but we only extract metadata (small)
- Estimated metadata size: ~200 columns × ~200 bytes = ~40KB per table
- Total additional memory: <100KB (negligible)

**Startup time**:
- Parsing VOTable XML may add 1-2 seconds to server startup
- Acceptable tradeoff for runtime performance
- Alternative: Pre-process metadata to JSON file (future optimization)

**Runtime performance**:
- Metadata lookup: O(1) HashMap access
- Building column_descriptions: O(n) where n = number of columns (~10-20)
- Negligible impact on request latency

---

### File Dependency Map

```
Metadata Loading (Server Startup):
  src/main.rs
    ↓ uses
  exo-core/src/metadata.rs
    ↓ reads
  data/exoplanets.vot
  data/stellarhosts.vot

Data Flow (Request):
  src/components/exoplanets_table.rs
    ↓ calls
  src/server/functions.rs::get_exoplanets_page()
    ↓ uses context
  ApiState (with metadata)
    ↓ calls
  src/server/common.rs::get_exoplanets_data()
    ↓ combines data + metadata
  TableData (with column_descriptions)
    ↓ renders
  src/table/table.rs::Table component
```

---

### Rollback Plan

If issues arise:
1. Set `column_descriptions: None` in server functions
2. Application will work without tooltips
3. Fix metadata loading and redeploy

No database schema changes required - purely in-memory changes.

**Potential future enhancements:**
- Add search/filter functionality to tables
- Add detailed view pages for individual planets/stars
- Add data visualizations (charts, graphs)
- Add export functionality (CSV, JSON)
- Add column visibility toggles
- Add more statistical insights to overview page
