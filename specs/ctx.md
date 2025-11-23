# Exoplanets Catalog: MVP Development Plan

This document outlines the plan for developing the Minimum Viable Product (MVP) of the Exoplanets Catalog application.

## 1. Data Handling

### 1.1. Data Format
The exoplanet data is in VOTable format. We will use the `votable` crate to parse this data in Rust. This crate seems to be the standard for handling VOTable files in the Rust ecosystem.

### 1.2. Data Acquisition
The project's `Justfile` already contains a command `download-stellarhosts` to download the necessary data from the NASA Exoplanet Archive and store it as `data/stellarhosts.vot`.

```bash
just download-stellarhosts
```

This command will be our primary way of fetching the dataset.

### 1.3. Data Storage and Access
For the MVP, we will follow the architecture outlined in `specs/overview.md` and load the entire dataset from the `.vot` file into memory on application startup.

-   We will define a Rust `struct` that represents a single stellar host/exoplanet record.
-   The data from the VOTable will be parsed into a `Vec<StellarHost>` (or a similar structure).
-   This `Vec` will be held in a shared state (e.g., an `Arc`) accessible by the Axum request handlers.

This in-memory approach will provide the fastest possible query performance for a dataset of this size. We can consider a database solution in the future if the data grows too large to fit comfortably in memory.

## 2. Backend (Axum)

The Axum backend will be responsible for serving the frontend and providing a data API.

### 2.1. API for Queries
The backend will expose a RESTful API endpoint (e.g., `/api/stellarhosts`) for the frontend to query the exoplanet data. This endpoint will support the following query parameters:

-   **Filtering**: Allow filtering by various fields (e.g., `pl_name`, `hostname`). For numeric fields, we can support range queries (e.g., `pl_orbper_min`, `pl_orbper_max`).
-   **Sorting**: Allow sorting by any of the data fields (e.g., `sort_by=pl_orbper&order=asc`).
-   **Pagination**: Implement pagination to handle the large number of records (e.g., `page=1&limit=50`).

The logic for these operations will be implemented in Rust, operating on the in-memory `Vec<StellarHost>`.

## 3. Frontend (Leptos)

The frontend will be a Single Page Application (SPA) built with Leptos.

### 3.1. Data Display
-   A primary view will display the exoplanet data in a tabular format.
-   This table will be populated by fetching data from the backend's `/api/stellarhosts` endpoint.

### 3.2. User Interaction
-   The UI will include controls for filtering, sorting, and navigating through the pages of data.
-   When the user interacts with these controls, the frontend will make new requests to the backend API with the appropriate query parameters and update the displayed data.

## 4. Implementation Steps

1.  **Add `votable` dependency**: Add the `votable` crate to `Cargo.toml`.
2.  **Define data structures**: Create the Rust structs to represent the stellar host data in `src/stellarhosts.rs`.
3.  **Implement data loading**: Write the logic in `src/stellarhosts.rs` to parse the `stellarhosts.vot` file into the in-memory data structures.
4.  **Create API endpoint**: In `src/main.rs`, create the Axum route for `/api/stellarhosts`.
5.  **Implement query logic**: Implement the filtering, sorting, and pagination logic for the API endpoint.
6.  **Develop frontend table view**: In `src/app.rs`, create the Leptos component for the data table.
7.  **Implement frontend controls**: Add the UI controls for filtering, sorting, and pagination and connect them to the backend API.
