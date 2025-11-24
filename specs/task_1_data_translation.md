## 1. Data Handling

This document outlines the strategy for parsing, storing, and querying the exoplanet data. The primary goal is to load the data into a high-performance, in-memory structure that supports complex analytical queries.

### 1.1. Data Source
The exoplanet data is provided as a VOTable file (`data/stellarhosts.vot`), acquired from the NASA Exoplanet Archive via the `just download-stellarhosts` command.

### 1.2. Core Technology
We will use two main Rust crates for this task:
-   **`votable`**: To parse the raw `stellarhosts.vot` file.
-   **`polars`**: To store the data in a `DataFrame` and perform all subsequent querying and analysis.

### 1.3. Data Loading and Storage Strategy
To achieve the best performance and memory efficiency, we will load the data directly from the VOTable file into a Polars DataFrame, bypassing any intermediate struct representations.

The process is as follows:
1.  **Parse VOTable Schema:** The `votable` crate will be used to read the header of the VOTable file. From this, we will extract the list of all `FIELD` definitions to get their `name` and `datatype`.
2.  **Initialize Column Buffers:** For each field in the schema, we will create a temporary, typed `Vec` (e.g., `Vec<Option<f64>>`) to act as a buffer for that column's data.
3.  **Populate Buffers:** We will iterate through the data rows of the VOTable. In each row, the value of each cell will be parsed and appended to its corresponding column buffer.
4.  **Build DataFrame:** After all rows have been processed, each column buffer will be converted into a Polars `Series`. The name for each `Series` will be the `name` extracted from the VOTable `FIELD` definition. These `Series` will then be combined to construct the final `DataFrame`.
5.  **Store DataFrame:** The completed `DataFrame` will be placed into a shared state manager (e.g., `Arc<DataFrame>`) to make it accessible to all parts of the application, such as Axum API handlers.

### 1.4. Querying and Analysis
All data queries and calculations will be performed on the in-memory Polars `DataFrame`.

-   **Column Access:** Columns will be accessed using the string names read from the VOTable header (e.g., `"st_teff"`, `"st_tefferr1"`, `"st_tefflim"`).
-   **Analytical Calculations:** The columnar structure of the DataFrame makes it trivial to perform calculations on related fields. For example, to calculate a stellar temperature's upper bound, we can directly add the value and error columns.

    *Conceptual Example:*
    ```rust
    // df is our DataFrame
    let result = df.lazy()
        .with_column(
            // Add a new column by combining two existing ones
            (col("st_teff") + col("st_tefferr1")).alias("st_teff_upper_bound")
        )
        .collect()?;
    ```
-   **API Integration:** Backend API endpoints will query this central `DataFrame` to perform any filtering, sorting, and aggregation required by the frontend, ensuring fast and efficient responses.

### 1.5. Schema Reference
The authoritative schema for the `stellarhosts` table could be retrieved from the NASA Exoplanet Archive's TAP service.

**TAP Query URL:**

```

https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=SELECT+column_name,description,datatype,unit+FROM+TAP_SCHEMA.columns+WHERE+table_name+%3D+'stellarhosts'&format=csv

```


