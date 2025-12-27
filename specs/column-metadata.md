# Column Metadata Mapping Spec

## Overview

Add comprehensive column metadata (descriptions, units, data types) to `exo-core` crate based on NASA Exoplanet Archive official documentation.

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

## Implementation Plan

### 1. Add XML parsing dependency

Add to `exo-core/Cargo.toml`:
```toml
[dependencies]
quick-xml = "0.32"
serde = { version = "1.0", features = ["derive"] }
```

### 2. Create `exo-core/src/metadata.rs`

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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

**Note**: We use `include_str!` to embed the VOTable files at compile time, so the metadata is always available without runtime file I/O.

### 3. Integration Points

**exo-core:**
- Add `pub mod metadata;` to `lib.rs`
- Expose metadata functions

**Frontend (exoplanets-catalog):**
- Import metadata from `exo_core::metadata`
- Pass to `Table` component as `column_descriptions` prop
- Table component renders tooltips with descriptions

### 4. Future Enhancements

- Cache parsed metadata (use `lazy_static` or `OnceLock`)
- Include more VOTable fields (ID, arraysize, UCD, etc.)
- Support for error column descriptions (err1, err2, lim fields)
- Validation that all displayed columns have metadata

## File Structure

```
exo-core/
  src/
    lib.rs              # Add: pub mod metadata;
    metadata.rs         # NEW: VOTable parser and metadata functions
data/
  exoplanets.vot       # Source of exoplanet metadata
  stellarhosts.vot     # Source of stellar host metadata
```

## Testing

- Unit test: Verify VOTable parser extracts all fields correctly
- Unit test: Verify all displayed columns have metadata
- Integration test: Verify metadata is accessible from exo-core
- Frontend test: Verify tooltips render correctly with descriptions

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
