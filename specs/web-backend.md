# Web Backend Specification: Axum Server

This document describes the Axum web server and server-side functionality of the exoplanets catalog web application.

## Overview

The web backend provides:
- **Axum HTTP server** for serving the application
- **REST API endpoints** for data access
- **Leptos server functions** for client-server communication
- **In-memory data loading** at startup for fast access
- **State management** for sharing data across requests

## Architecture

```
src/
├── main.rs           # Server startup and configuration
├── server/
│   ├── mod.rs        # Module declarations
│   ├── handlers.rs   # REST API handlers and ApiState
│   └── functions.rs  # Leptos server functions
└── tables/           # Data processing (uses local copy)
```

## Server Startup (main.rs)

### Entry Point

```rust
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    start_server().await;
}
```

### Server Initialization

The `start_server()` function:

1. **Load data at startup**
```rust
let stellarhosts_df = match common::load_parquet("data/stellarhosts.parquet", None) {
    Ok(df) => Arc::new(df),
    Err(e) => panic!("Failed to load stellarhosts data: {}", e),
};

let exoplanets_df = match common::load_parquet("data/exoplanets.parquet", None) {
    Ok(df) => Arc::new(df),
    Err(e) => panic!("Failed to load exoplanets data: {}", e),
};
```

2. **Create shared state**
```rust
let api_state = ApiState {
    stellarhosts_df,
    exoplanets_df,
};
```

3. **Build router**
```rust
let app = Router::new()
    .leptos_routes(&leptos_options, routes, {
        let api_state = api_state.clone();
        move || {
            provide_context(api_state.clone());
            shell(leptos_options.clone())
        }
    })
    .nest_service("/api", server::api_routes(api_state))
    .fallback(leptos_axum::file_and_error_handler(shell))
    .with_state(leptos_options);
```

4. **Start server**
```rust
let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
println!("listening on http://{}", &addr);
axum::serve(listener, app.into_make_service())
    .await
    .unwrap();
```

### Key Design Decisions

- **Data loaded once**: Files read at startup, not per-request
- **Arc<DataFrame>**: Shared ownership, no cloning data
- **Dual state sharing**: ApiState provided to both Axum (`.with_state()`) and Leptos (`provide_context()`)
- **Panic on load failure**: Missing data files prevent server start (fail-fast)

## API State (server/handlers.rs)

### ApiState Structure

```rust
#[derive(Debug, Clone)]
pub struct ApiState {
    pub stellarhosts_df: Arc<DataFrame>,
    pub exoplanets_df: Arc<DataFrame>,
}
```

**Thread Safety:**
- `Arc<DataFrame>` allows sharing across async tasks
- `Clone` creates new Arc pointers (cheap, no data copy)
- DataFrames are immutable (read-only access)

### Query Parameters

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryParams {
    // Pagination
    pub page: Option<usize>,
    pub limit: Option<usize>,

    // Sorting
    pub sort_by: Option<String>,
    pub order: Option<String>, // "asc" or "desc"

    // Text filters
    pub hostname: Option<String>,
    pub pl_name: Option<String>,

    // Numeric range filters
    pub sy_dist_min: Option<f64>,
    pub sy_dist_max: Option<f64>,
    pub st_teff_min: Option<f64>,
    pub st_teff_max: Option<f64>,
    pub pl_orbper_min: Option<f64>,
    pub pl_orbper_max: Option<f64>,
    pub pl_rade_min: Option<f64>,
    pub pl_rade_max: Option<f64>,
    pub pl_masse_min: Option<f64>,
    pub pl_masse_max: Option<f64>,
}
```

### API Response Structure

```rust
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub limit: usize,
    pub filters: QueryParams,
}
```

## REST API Endpoints

### Route Setup

```rust
pub fn api_routes(state: ApiState) -> Router {
    Router::new()
        .route("/stellarhosts", get(get_stellarhosts))
        .route("/exoplanets", get(get_exoplanets))
        .route("/stellarhosts/schema", get(get_stellarhosts_schema))
        .route("/exoplanets/schema", get(get_exoplanets_schema))
        .with_state(state)
}
```

### 1. GET /api/stellarhosts

Get stellar hosts data with filtering, sorting, and pagination.

**Handler:**
```rust
pub async fn get_stellarhosts(
    State(state): State<ApiState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<ApiResponse<Value>>, StatusCode>
```

**Query Parameters:**
- `page` - Page number (default: 1)
- `limit` - Items per page (default: 50)
- `sort_by` - Column name to sort by
- `order` - Sort direction ("asc" or "desc")
- `hostname` - Filter by hostname (partial match)
- `st_teff_min`, `st_teff_max` - Temperature range
- `sy_dist_min`, `sy_dist_max` - Distance range

**Example Request:**
```
GET /api/stellarhosts?page=1&limit=20&st_teff_min=5000&st_teff_max=6000&sort_by=st_teff&order=desc
```

**Response:**
```json
{
  "data": [
    {
      "hostname": "Kepler-452",
      "st_teff": 5757.0,
      "st_mass": 1.04,
      "st_rad": 1.11,
      ...
    },
    ...
  ],
  "total": 1523,
  "page": 1,
  "limit": 20,
  "filters": {
    "st_teff_min": 5000.0,
    "st_teff_max": 6000.0,
    ...
  }
}
```

**Processing Steps:**
1. Clone DataFrame from state
2. Apply filters (`apply_stellarhosts_filters`)
3. Count total results
4. Apply sorting (`apply_sorting`)
5. Apply pagination (`apply_pagination`)
6. Convert to JSON (`dataframe_to_json`)
7. Return response

### 2. GET /api/exoplanets

Get exoplanets data with filtering, sorting, and pagination.

**Handler:**
```rust
pub async fn get_exoplanets(
    State(state): State<ApiState>,
    Query(params): Query<QueryParams>,
) -> Result<Json<ApiResponse<Value>>, StatusCode>
```

**Query Parameters:**
- Same pagination/sorting as `/stellarhosts`
- `pl_name` - Filter by planet name
- `pl_orbper_min`, `pl_orbper_max` - Orbital period range
- `pl_rade_min`, `pl_rade_max` - Planet radius range
- `pl_masse_min`, `pl_masse_max` - Planet mass range

**Example Request:**
```
GET /api/exoplanets?pl_rade_min=0.8&pl_rade_max=1.2&sort_by=disc_year&order=desc
```

**Response:**
Similar structure to `/stellarhosts` response.

### 3. GET /api/stellarhosts/schema

Get column names and types for stellarhosts dataset.

**Response:**
```json
{
  "columns": [
    {"name": "hostname", "type": "String"},
    {"name": "st_teff", "type": "Float64"},
    {"name": "st_mass", "type": "Float64"},
    ...
  ]
}
```

### 4. GET /api/exoplanets/schema

Get column names and types for exoplanets dataset.

**Response:**
Similar to `/stellarhosts/schema`.

## Helper Functions

### Filtering

```rust
fn apply_stellarhosts_filters(
    df: DataFrame,
    params: &QueryParams,
) -> Result<DataFrame, StatusCode>
```

Applies filters based on query parameters:
- Text matching (hostname, etc.)
- Numeric ranges (temperature, distance, etc.)
- Null handling

### Sorting

```rust
fn apply_sorting(
    df: DataFrame,
    sort_by: &str,
    order: Option<&str>,
) -> Result<DataFrame, StatusCode>
```

Sorts DataFrame by specified column in ascending or descending order.

### Pagination

```rust
fn apply_pagination(
    df: DataFrame,
    page: Option<usize>,
    limit: Option<usize>,
) -> Result<DataFrame, StatusCode>
```

Slices DataFrame to return requested page:
- Default limit: 50 rows
- Default page: 1
- Calculates offset: `(page - 1) * limit`

### JSON Conversion

```rust
fn dataframe_to_json(df: &DataFrame) -> Result<Vec<Value>, StatusCode>
```

Converts DataFrame rows to JSON objects:
- Each row becomes a JSON object
- Column names become keys
- Handles null values appropriately

## Leptos Server Functions (server/functions.rs)

### What Are Server Functions?

Leptos server functions allow calling server-side code from the client:
- Decorated with `#[server]` macro
- Called like async functions from client
- Automatically serialize/deserialize data
- Type-safe across client-server boundary

### get_stats Server Function

```rust
#[server(GetStats, "/api")]
pub async fn get_stats() -> Result<DataStats, ServerFnError>
```

**Purpose:**
Calculate overview statistics for the homepage.

**Implementation:**
```rust
// 1. Get ApiState from context
let state = expect_context::<ApiState>();

// 2. Calculate statistics using exo-core aggregations
let (stellarhosts_total, exoplanets_total) =
    aggregation::get_total_counts(&state.stellarhosts_df, &state.exoplanets_df);

let avg_stellar_temp =
    aggregation::get_avg_temperature(&state.stellarhosts_df).unwrap_or(0.0);

let avg_stellar_distance =
    aggregation::get_avg_distance(&state.stellarhosts_df).unwrap_or(0.0);

let discovery_methods =
    aggregation::get_discovery_methods(&state.exoplanets_df, 10);

let planet_size_categories =
    aggregation::get_planet_size_categories(&state.exoplanets_df);

// 3. Return aggregated data
Ok(DataStats {
    stellarhosts_total,
    exoplanets_total,
    avg_stellar_temp,
    avg_stellar_distance,
    discovery_methods,
    planet_size_categories,
})
```

**Data Structure:**
```rust
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DataStats {
    pub stellarhosts_total: usize,
    pub exoplanets_total: usize,
    pub avg_stellar_temp: f64,
    pub avg_stellar_distance: f64,
    pub discovery_methods: Vec<(String, usize)>,
    pub planet_size_categories: Vec<(String, usize)>,
}
```

**Client Usage:**
```rust
// In Leptos component
let stats_resource = Resource::new(
    move || (),
    move |_| async move { get_stats().await },
);
```

### Why Server Functions?

**Advantages over REST:**
- No HTTP self-requests from server
- Direct data access via context
- Type-safe API contract
- Automatic serialization
- Integrated with Leptos reactivity

## Error Handling

### REST API Errors

Return appropriate HTTP status codes:
```rust
Result<Json<ApiResponse<T>>, StatusCode>
```

Common errors:
- `400 Bad Request` - Invalid query parameters
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server-side errors

### Server Function Errors

Return `ServerFnError`:
```rust
Result<DataStats, ServerFnError>
```

Errors automatically propagated to client.

## Configuration

### Server Address

Configured in `Cargo.toml`:
```toml
[package.metadata.leptos]
site-addr = "127.0.0.1:3000"
```

### Data File Paths

Hardcoded in `main.rs`:
```rust
common::load_parquet("data/stellarhosts.parquet", None)
common::load_parquet("data/exoplanets.parquet", None)
```

**Future:** Make configurable via environment variables.

## Performance Considerations

1. **In-Memory Data**: Fast access, but uses memory
2. **Arc<DataFrame>**: Cheap cloning via reference counting
3. **No Disk I/O**: Per-request (loaded once at startup)
4. **Polars Operations**: Highly optimized DataFrame operations
5. **Async Handlers**: Non-blocking I/O with Tokio

## Development vs Production

**Development:**
```bash
cargo leptos watch
# Hot reload enabled
# Debug logging
```

**Production:**
```bash
cargo leptos build --release
# Optimized binary
# WASM optimized
```

## Future Enhancements

- Environment-based configuration
- Database backend option (PostgreSQL, etc.)
- Caching layer (Redis)
- GraphQL API alternative
- WebSocket support for real-time updates
- Authentication and authorization
- Rate limiting
- Compression middleware
- CORS configuration
