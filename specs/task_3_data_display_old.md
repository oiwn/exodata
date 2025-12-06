## 3. Frontend (Leptos)

The frontend will be a Single Page Application (SPA) built with Leptos.

### 3.1. Data Display
-   A primary view will display the exoplanet data in a tabular format.
-   This table will be populated by fetching data from the backend's `/api/stellarhosts` endpoint.

### 3.2. User Interaction
-   The UI will include controls for filtering, sorting, and navigating through the pages of data.
-   When the user interacts with these controls, the frontend will make new requests to the backend API with the appropriate query parameters and update the displayed data.
