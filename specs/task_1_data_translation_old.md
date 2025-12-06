## 2. Backend (Axum)

The Axum backend will be responsible for serving the frontend and providing a data API.

### 2.1. API for Queries
The backend will expose a RESTful API endpoint (e.g., `/api/stellarhosts`) for the frontend to query the exoplanet data. This endpoint will support the following query parameters:

-   **Filtering**: Allow filtering by various fields (e.g., `pl_name`, `hostname`). For numeric fields, we can support range queries (e.g., `pl_orbper_min`, `pl_orbper_max`).
-   **Sorting**: Allow sorting by any of the data fields (e.g., `sort_by=pl_orbper&order=asc`).
-   **Pagination**: Implement pagination to handle the large number of records (e.g., `page=1&limit=50`).

The logic for these operations will be implemented in Rust, operating on the in-memory `Vec<StellarHost>`.
