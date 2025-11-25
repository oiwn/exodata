## 1. Data Handling

This document outlines the strategy for parsing, storing, and querying the exoplanet data. The primary goal is to load the data into a high-performance, in-memory structure that supports complex analytical queries.

### 1.1. Data Source
The exoplanet data is provided as a VOTable file (`data/stellarhosts.vot`), acquired from the NASA Exoplanet Archive via the `just download-stellarhosts` command.

### 1.2. Core Technology
We use two main Rust crates for this task:
-   **`votable`**: To parse the raw `stellarhosts.vot` file.
-   **`polars`**: To store the data in a `DataFrame` and perform all subsequent querying and analysis.

### 1.3. Data Loading and Storage Strategy - COMPLETED ✓

The implementation follows the specified approach:
1.  **Parse VOTable Schema:** The `votable` crate is used to read the header of the VOTable file, extracting `FIELD` definitions to get their `name` and `datatype`.
2.  **Initialize Column Buffers:** For each field in the schema, we create typed `Vec` buffers based on the field's datatype (e.g., `Vec<Option<f64>>` for Double, `Vec<Option<String>>` for CharASCII/CharUnicode).
3.  **Populate Buffers:** We iterate through the data rows of the VOTable, parsing each cell value and appending it to the corresponding column buffer.
4.  **Build DataFrame:** After processing all rows, each column buffer is converted into a Polars `Series` with the appropriate name, then combined to create the final `DataFrame`.
5.  **Store DataFrame:** The completed `DataFrame` is ready to be stored in shared state or used for queries.

The implementation can be tested using:
```bash
cargo run -- check
```

### 1.4. Querying and Analysis - PARTIALLY COMPLETED

All data queries and calculations can be performed on the in-memory Polars `DataFrame`.

-   **Column Access:** Columns are accessed using the string names read from the VOTable header (e.g., `"st_teff"`, `"st_tefferr1"`, `"st_tefflim"`).
-   **Analytical Calculations:** The columnar structure of the DataFrame makes it trivial to perform calculations on related fields.

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

**TODO:**
- [ ] Implement backend API endpoints to query the central `DataFrame`
- [ ] Add specific analytical calculations as needed by the frontend

### 1.5. Schema Reference

The authoritative schema for the `stellarhosts` table can be retrieved from the NASA Exoplanet Archive's TAP service.

**TAP Query URL:**
```
https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=SELECT+column_name,description,datatype,unit+FROM+TAP_SCHEMA.columns+WHERE+table_name+%3D+'stellarhosts'&format=csv
```

### 1.6. Next Steps

The data loading and parsing part of Task 1 is complete. The next focus area should be:

1. **CLI Data Exploration Tools**: Create CLI commands for data exploration and analysis
   - Schema inspection
   - Data filtering and querying
   - Basic statistics
   - Data export capabilities

2. **Backend API Integration**: Expose the DataFrame through API endpoints for frontend consumption
   - RESTful endpoints for data queries
   - Filtering and sorting capabilities
   - Pagination support

3. **Frontend Integration**: Connect frontend to the backend API for interactive data browsing


