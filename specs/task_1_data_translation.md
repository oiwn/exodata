## 1. Data Handling

This document outlines the strategy for parsing, storing, and querying exoplanet and stellar host data. The primary goal is to load data into a high-performance, in-memory structure that supports complex analytical queries.

### 1.1. Data Sources
The exoplanet and stellar host data are provided as VOTable files, acquired from NASA Exoplanet Archive via Just commands:
- `data/stellarhosts.vot` (167MB) - Information about stars hosting exoplanets
- `data/exoplanets.vot` (394MB) - Confirmed exoplanets data from `ps` table

### 1.2. Core Technology
We use two main Rust crates for this task:
-   **`votable`**: To parse raw VOTable files.
-   **`polars`**: To store data in a `DataFrame` and perform all subsequent querying and analysis.

### 1.3. Data Loading and Storage Strategy - COMPLETED ✓

The implementation follows the specified approach:
1.  **Parse VOTable Schema:** The `votable` crate is used to read the header of the VOTable file, extracting `FIELD` definitions to get their `name` and `datatype`.
2.  **Initialize Column Buffers:** For each field in the schema, we create typed `Vec` buffers based on the field's datatype (e.g., `Vec<Option<f64>>` for Double, `Vec<Option<String>>` for CharASCII/CharUnicode).
3.  **Populate Buffers:** We iterate through the data rows of the VOTable, parsing each cell value and appending it to the corresponding column buffer.
4.  **Build DataFrame:** After processing all rows, each column buffer is converted into a Polars `Series` with the appropriate name, then combined to create the final `DataFrame`.
5.  **Store DataFrame:** The completed `DataFrame` is ready to be stored in shared state or used for queries.

The implementation can be tested using:
```bash
cargo run -- view-fields data/stellarhosts.vot
cargo run -- view-fields data/exoplanets.vot
```

### 1.4. CLI Data Exploration Tools - PARTIALLY COMPLETED (STELLARHOSTS ONLY)

All data queries and calculations can be performed on in-memory Polars `DataFrame`.

-   **Column Access:** Columns are accessed using string names read from VOTable header (e.g., `"st_teff"`, `"pl_name"`, `"pl_masse"`).
-   **Current CLI Commands (stellarhosts only):** 
    - `view-samples` - View data samples with customizable column categories
      - Options: `--limit`, `--category` (basic, position, stellar, photometry)
      - Example: `cargo run -- view-samples --limit 5 --category stellar`
    - `view-stats` - Display basic statistics and distributions
      - Shows mean, median, std dev, min/max for key columns
      - Includes histogram visualizations for temperature, mass, and radius
    - `view-fields` - Print all available fields in VOTable
    - **Performance:** Optimized loading function (`load_data_with_limit`) for faster data exploration
      - Only loads required rows instead of entire dataset
      - Reduced load time from several seconds to <1s for sample viewing

**COMPLETED:**
- [x] CLI commands for stellarhosts data exploration
- [x] Column categorization for stellarhosts
- [x] Basic statistics for stellarhosts
- [x] Performance optimization with partial loading

**TODO:**
- [ ] Add support for exoplanets data table
- [ ] Create appropriate column categories for exoplanets data
- [ ] Implement backend API endpoints to query both DataFrames
- [ ] Add specific analytical calculations as needed by frontend

Example conceptual calculation:
```rust
// df is our DataFrame
let result = df.lazy()
    .with_column(
        // Add a new column by combining two existing ones
        (col("st_teff") + col("st_tefferr1")).alias("st_teff_upper_bound")
    )
    .collect()?;
```

### 1.5. Schema Reference

The authoritative schemas for tables can be retrieved from NASA Exoplanet Archive's TAP service.

**TAP Query URLs:**
```
# For stellarhosts
https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=SELECT+column_name,description,datatype,unit+FROM+TAP_SCHEMA.columns+WHERE+table_name+%3D+'stellarhosts'&format=csv

# For exoplanets (ps table)
https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=SELECT+column_name,description,datatype,unit+FROM+TAP_SCHEMA.columns+WHERE+table_name+%3D+'ps'&format=csv
```

### 1.6. Next Steps

The data loading for stellarhosts is complete, and CLI tools for stellarhosts are implemented. Next focus areas:

1. **Extend CLI to Support Exoplanets Table**:
   - Create `exoplanets.rs` module similar to `stellarhosts.rs`
   - Implement data loading function for exoplanets VOTable
   - Define appropriate column categories for exoplanets data
   - Update CLI commands to accept table parameter

2. **Backend API Integration**: Expose both DataFrames through API endpoints for frontend consumption
   - RESTful endpoints for both stellarhosts and exoplanets
   - Filtering and sorting capabilities for both tables
   - Pagination support

3. **Frontend Integration**: Connect frontend to backend API for interactive data browsing