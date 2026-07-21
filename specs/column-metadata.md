# Column Metadata Mapping Spec

## Overview

Add comprehensive column metadata (descriptions, units, data types) based on NASA Exoplanet Archive official documentation. `exo-types` owns the shared serializable `ColumnMetadata` type; `exo-core` owns metadata parsing, persistence, and presentation helpers.

**Status**: ✅ **COMPLETED** - Full pipeline implemented (VOTable → TOML → Server → API)

**Summary**:
- ✅ Metadata extraction from VOTable files
- ✅ TOML persistence (`data/*-metadata.toml`)
- ✅ Server-side loading and filtering
- ✅ API integration (metadata in `TableData` response)
- ⏳ Frontend integration (next step)

## Problem

Currently, the table UI displays raw column names like `pl_orbper`, `st_teff`, etc., which are not user-friendly. Users need to understand what these columns represent without referring to external documentation.

## Solution

Create a centralized column metadata module in `exo-core` that:
1. Provides official NASA Exoplanet Archive descriptions for all columns
2. Includes units and data types
3. Can be easily queried by the frontend for tooltips/documentation

## Data Source

**VOTable Files** (Recommended Approach):
- The `.vot` files already contain complete metadata for all columns
- Location: `data/exoplanets.vot`, `data/stellarhosts.vot`
- Each `<FIELD>` element contains:
  - `name` attribute: Column name (e.g., "pl_rade")
  - `<DESCRIPTION>`: Human-readable description
  - `unit` attribute: Unit of measurement (e.g., "Rearth", "day")
  - `datatype` attribute: Data type (e.g., "double", "char", "int")

**Example from VOTable:**
```xml
<FIELD ID="pl_rade" datatype="double" name="pl_rade" unit="Rearth"/>
<FIELD ID="pl_radestr" arraysize="*" datatype="char" name="pl_radestr" unit="Rearth">
  <DESCRIPTION><![CDATA[ Planet Radius [Earth Radius] ]]></DESCRIPTION>
</FIELD>

<FIELD ID="pl_bmasse" datatype="double" name="pl_bmasse" unit="Mearth"/>
<FIELD ID="pl_bmassestr" arraysize="*" datatype="char" name="pl_bmassestr" unit="Mearth">
  <DESCRIPTION><![CDATA[ Planet Mass or Mass*sin(i) [Earth Mass] ]]></DESCRIPTION>
</FIELD>
```

**Fallback:**
- NASA Exoplanet Archive Documentation: https://exoplanetarchive.ipac.caltech.edu/docs/API_PS_columns.html

## Implementation Status

✅ **COMPLETED**: Full metadata integration pipeline (VOTable → TOML → Server → API)

### Implementation Details

### 1. Created `exo-core/src/metadata.rs` ✅

Implemented in `/crates/exo-core/src/metadata.rs` with the following functions:

**Core Metadata Functions:**

```rust
// Parse VOTable and extract all column metadata
pub fn parse_votable_metadata(vot_path: &str) -> Result<HashMap<String, ColumnMetadata>, String>

// Get metadata for exoplanets
pub fn get_exoplanets_metadata(vot_path: &str) -> HashMap<String, ColumnMetadata>

// Get metadata for stellar hosts
pub fn get_stellarhosts_metadata(vot_path: &str) -> HashMap<String, ColumnMetadata>

// Print metadata in human-readable format
pub fn print_metadata(metadata: &HashMap<String, ColumnMetadata>)

// Get metadata for specific columns only
pub fn get_columns_metadata(
    all_metadata: &HashMap<String, ColumnMetadata>,
    column_names: &[&str],
) -> HashMap<String, String>

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub datatype: String,
}

/// Parse VOTable XML and extract column metadata
pub fn parse_votable_metadata(vot_content: &str) -> Result<HashMap<String, ColumnMetadata>, String> {
    // Parse XML and extract FIELD elements
    // Map name -> ColumnMetadata
}

/// Get metadata for exoplanets columns (from exoplanets.vot)
pub fn get_exoplanets_metadata() -> HashMap<String, ColumnMetadata> {
    let vot_content = include_str!("../../data/exoplanets.vot");
    parse_votable_metadata(vot_content).unwrap_or_default()
}

/// Get metadata for stellar hosts columns (from stellarhosts.vot)
pub fn get_stellarhosts_metadata() -> HashMap<String, ColumnMetadata> {
    let vot_content = include_str!("../../data/stellarhosts.vot");
    parse_votable_metadata(vot_content).unwrap_or_default()
}
```

### 2. Added CLI Command ✅

Added `ViewMetadata` command to exo-cli (`/crates/exo-cli/src/main.rs`):

```bash
# View all metadata from a VOTable file
exodata dev view-metadata --path data/exoplanets.vot

# View metadata for specific columns
exodata dev view-metadata --path data/exoplanets.vot --columns "pl_name,pl_orbper,pl_rade"
```

### 3. TOML Metadata Generation ✅

**Location**: `crates/exo-core/src/tables/conversion.rs`

The `convert-raw-files` CLI command now:
1. Parses VOTable metadata using `parse_votable_metadata()`
2. Saves metadata to TOML files alongside parquet files
3. Generates:
   - `data/exoplanets-metadata.toml` (25KB, 176 columns)
   - `data/stellarhosts-metadata.toml` (8.7KB, 78 columns)

**TOML Format:**
```toml
[[column]]
name = "pl_orbper"
description = "Orbital Period"
unit = "day"
datatype = "Double"

[[column]]
name = "hostname"
description = "Host Name"
datatype = "CharASCII"
```

**Usage:**
```bash
cargo run -p exodata -- dev convert-raw-files --data-dir data
# Generates .parquet and .toml files
```

### 4. Server Integration ✅

**Files Modified:**
- `src/server/handlers.rs`: Extended `ApiState` with metadata fields
- `src/main.rs`: Load metadata from TOML files at startup
- `src/server/common.rs`: Updated business logic to accept and filter metadata
- `src/server/functions.rs`: Extended `TableData` struct with metadata field

**Data Flow:**
```
Server Startup:
  TOML files → load_metadata_toml() → Arc<HashMap<String, ColumnMetadata>>
                                              ↓
                                         ApiState.{exoplanets,stellarhosts}_metadata

API Request:
  Server Function → Business Logic → Filter metadata to selected columns
                                              ↓
                                    TableData.metadata field
```

**API Response Structure:**
```rust
pub struct TableData {
    pub rows: Vec<Value>,
    pub columns: Vec<String>,
    pub total: usize,
    pub total_all: usize,
    pub page: usize,
    pub limit: usize,
    pub metadata: HashMap<String, ColumnMetadata>,  // NEW: Full metadata for displayed columns
}
```

### 5. Next: Frontend Integration ⏳

**TODO**: Wire metadata to Table component tooltips

The Table component already supports `column_descriptions` prop. The metadata is now available in the API response.

**Integration steps:**
1. Update table components to read `data.metadata` from API response
2. Convert `HashMap<String, ColumnMetadata>` to `HashMap<String, String>` format expected by Table
3. Pass to `<Table column_descriptions={...} />` component
4. Tooltips will display automatically on column headers

### 6. Future Enhancements

- Include more VOTable fields (ID, arraysize, UCD, etc.)
- Support for error column descriptions (err1, err2, lim fields)
- Validation that all displayed columns have metadata
- Column selection via query parameter (specify which columns to display)

## File Structure

```
crates/exo-core/
  src/
    lib.rs                      # pub mod metadata;
    metadata.rs                 # ✅ VOTable parser, TOML save/load, metadata functions
    tables/
      conversion.rs             # ✅ UPDATED: Generates TOML files during conversion
  Cargo.toml                    # ✅ Dependencies: toml = "0.9", serde

data/
  exoplanets.vot                # Source of exoplanet metadata
  stellarhosts.vot              # Source of stellar host metadata
  exoplanets-metadata.toml      # ✅ NEW: Generated metadata (25KB, 176 columns)
  stellarhosts-metadata.toml    # ✅ NEW: Generated metadata (8.7KB, 78 columns)

src/
  main.rs                       # ✅ UPDATED: Load metadata at startup
  server/
    handlers.rs                 # ✅ UPDATED: ApiState with metadata fields
    common.rs                   # ✅ UPDATED: Business logic accepts/returns metadata
    functions.rs                # ✅ UPDATED: TableData includes metadata field
    tests.rs                    # ✅ UPDATED: Tests use empty metadata
```

## Testing

✅ **All tests passing** (11/11)

**Unit Tests:**
- ✅ `test_parse_votable_metadata` - Verifies VOTable parser doesn't panic
- ✅ `test_get_columns_metadata` - Verifies column filtering and unit formatting
- ✅ `test_save_and_load_metadata_toml` - Verifies TOML round-trip serialization
- ✅ `test_get_stellarhosts_data_pagination` - Verifies pagination with metadata
- ✅ `test_get_stellarhosts_data_sorting` - Verifies sorting with metadata

**Integration Tests:**
- ✅ REST API tests verify ApiState with metadata fields
- ✅ Server function tests verify metadata flows through pipeline

**Manual Verification:**
```bash
# Generate TOML files
cargo run -p exodata -- dev convert-raw-files --data-dir data

# Verify TOML format
head -30 data/exoplanets-metadata.toml

# Run server and check API response includes metadata
cargo leptos watch
# Visit /api endpoint and verify TableData.metadata field
```

## Benefits

1. **User Experience**: Clear descriptions on hover for all columns
2. **Automatic**: Metadata comes directly from official VOTable files
3. **Maintainability**: Single source of truth (VOTable files)
4. **No Manual Work**: No need to manually maintain column descriptions
5. **Always In Sync**: Metadata automatically matches the data structure
6. **Official**: Uses NASA Exoplanet Archive's official metadata

## Example Output

When hovering over column headers, users will see tooltips like:

- **pl_orbper**: "Orbital Period [day]"
- **pl_rade**: "Planet Radius [Earth Radius]"
- **pl_bmasse**: "Planet Mass or Mass*sin(i) [Earth Mass]"
- **st_teff**: (Description from VOTable)
- **sy_pnum**: (Description from VOTable)

## References

- [VOTable Format Specification](http://www.ivoa.net/documents/VOTable/)
- [NASA Exoplanet Archive](https://exoplanetarchive.ipac.caltech.edu/)
- [NASA Exoplanet Archive Column Definitions](https://exoplanetarchive.ipac.caltech.edu/docs/API_PS_columns.html)
