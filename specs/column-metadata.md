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

**Official NASA Exoplanet Archive Documentation:**
- Planetary Systems columns: https://exoplanetarchive.ipac.caltech.edu/docs/API_PS_columns.html
- Alternative: Extract from VOT (VOTable) files metadata

## Implementation Plan

### 1. Create `exo-core/src/metadata.rs`

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub unit: Option<String>,
    pub data_type: String,
}

/// Get metadata for all planetary system columns
pub fn get_planet_columns_metadata() -> HashMap<String, ColumnMetadata> {
    // ...
}

/// Get metadata for all stellar host columns
pub fn get_stellar_columns_metadata() -> HashMap<String, ColumnMetadata> {
    // ...
}

/// Get metadata for a specific column
pub fn get_column_metadata(column_name: &str) -> Option<ColumnMetadata> {
    // ...
}
```

### 2. Key Columns to Include

**Exoplanet Columns:**
- `pl_name` - Planet Name
- `hostname` - Host Star Name
- `discoverymethod` - Discovery Method
- `disc_year` - Discovery Year
- `pl_orbper` - Orbital Period (days)
- `pl_rade` - Planet Radius (Earth radii)
- `pl_bmasse` - Planet Mass (Earth masses)
- ... (all columns from VOT file)

**Stellar Host Columns:**
- `hostname` - Star Name
- `sy_dist` - System Distance (parsecs)
- `st_teff` - Stellar Temperature (Kelvin)
- `st_mass` - Stellar Mass (Solar masses)
- `sy_pnum` - Number of Planets
- ... (all columns from VOT file)

### 3. Column Metadata Structure

From NASA Exoplanet Archive documentation:

| Column | Display Name | Description | Unit |
|--------|--------------|-------------|------|
| pl_name | Planet Name | Planet name most commonly used in the literature | - |
| hostname | Star Name | Stellar name most commonly used in the literature | - |
| discoverymethod | Discovery Method | Method by which the planet was first identified | - |
| disc_year | Discovery Year | Year the planet was discovered | - |
| pl_orbper | Orbital Period | Time the planet takes to make a complete orbit around the host star | days |
| pl_rade | Planet Radius | Length of a line segment from the center of the planet to its surface | R⊕ (Earth Radius) |
| pl_bmasse | Planet Mass | Best planet mass estimate available | M⊕ (Earth Mass) |
| sy_dist | Distance | Distance to the planetary system | parsecs (pc) |
| st_teff | Temperature | Temperature of the star as modeled by a black body | Kelvin (K) |
| st_mass | Stellar Mass | Amount of matter contained in the star | M☉ (Solar mass) |
| sy_pnum | Planets | Number of confirmed planets in the planetary system | - |

### 4. Integration Points

**exo-core:**
- Add `pub mod metadata;` to `lib.rs`
- Expose metadata functions

**Frontend (exoplanets-catalog):**
- Import metadata from `exo_core::metadata`
- Pass to `Table` component as `column_descriptions` prop
- Table component renders tooltips with descriptions

### 5. Future Enhancements

- Extract metadata directly from VOT XML files
- Include more fields (precision, provenance, etc.)
- Support for different languages/locales
- Validation that all columns in data have metadata

## File Structure

```
exo-core/
  src/
    lib.rs              # Add: pub mod metadata;
    metadata.rs         # NEW: Column metadata module
    metadata/
      planets.rs        # NEW: Planet column metadata
      stellar.rs        # NEW: Stellar column metadata
```

## Testing

- Unit tests to verify all displayed columns have metadata
- Integration test to verify metadata matches VOT file columns
- Frontend test to verify tooltips render correctly

## Benefits

1. **User Experience**: Clear descriptions on hover
2. **Documentation**: Self-documenting data fields
3. **Maintainability**: Single source of truth for column metadata
4. **Extensibility**: Easy to add new columns or update descriptions
5. **Official**: Uses NASA's official documentation

## References

- [NASA Exoplanet Archive Column Definitions](https://exoplanetarchive.ipac.caltech.edu/docs/API_PS_columns.html)
- [VOTable Format Specification](http://www.ivoa.net/documents/VOTable/)
