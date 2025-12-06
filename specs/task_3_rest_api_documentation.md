## 3. REST API with OpenAPI Documentation

This document outlines the implementation specification for adding comprehensive OpenAPI/Swagger documentation to the Axum REST API using the `utoipa` crate.

### 3.1. Goal

Provide a fully documented, discoverable REST API for external consumers with interactive Swagger UI, enabling third-party integrations and API exploration.

### 3.2. Scope

#### 3.2.1. What's Included
- Add `utoipa` annotations to existing REST API endpoints
- Generate OpenAPI 3.0 specification automatically
- Serve Swagger UI at `/api/docs`
- Serve OpenAPI JSON spec at `/api/openapi.json`
- Document request/response schemas
- Document query parameters for filtering and pagination
- Add API metadata (title, version, description)

#### 3.2.2. What's NOT Included
- ❌ Authentication/authorization
- ❌ Rate limiting
- ❌ API versioning (v1, v2, etc.)
- ❌ CORS configuration (defer to deployment)
- ❌ New endpoints (only document existing ones)

### 3.3. API Endpoints to Document

#### 3.3.1. Data Endpoints
```
GET /api/stellarhosts
GET /api/exoplanets
GET /api/stats
GET /api/stellarhosts/schema
GET /api/exoplanets/schema
```

#### 3.3.2. Documentation Endpoints
```
GET /api/docs              # Swagger UI (HTML)
GET /api/openapi.json      # OpenAPI spec (JSON)
```

### 3.4. Documentation Requirements

#### 3.4.1. Endpoint Documentation
Each endpoint must include:
- Summary (one-line description)
- Detailed description
- Query parameters with types and descriptions
- Response schema with examples
- Error responses (400, 404, 500)

#### 3.4.2. Schema Documentation
All data structures must be documented:
- `QueryParams` - filtering, sorting, pagination parameters
- `ApiResponse<T>` - paginated response wrapper
- `OverviewStats` - aggregated statistics
- `SchemaInfo` - column metadata

#### 3.4.3. Example Responses
Provide realistic examples for:
- Single stellar host record
- Single exoplanet record
- Paginated response structure
- Statistics aggregation
- Schema metadata

### 3.5. Technical Implementation

#### 3.5.1. Dependencies
Add to `Cargo.toml`:
```toml
utoipa = { version = "5", features = ["axum"] }
utoipa-swagger-ui = { version = "8", features = ["axum"] }
```

#### 3.5.2. Annotations Required
- `#[derive(utoipa::ToSchema)]` on response structs
- `#[utoipa::path(...)]` on handler functions
- `#[derive(utoipa::IntoParams)]` on query parameter structs

#### 3.5.3. OpenAPI Configuration
```rust
// Conceptual structure
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Exoplanet Catalog API",
        version = "1.0.0",
        description = "REST API for querying exoplanet and stellar host data"
    ),
    paths(
        get_stellarhosts,
        get_exoplanets,
        get_stats,
        // ...
    ),
    components(
        schemas(QueryParams, ApiResponse, OverviewStats, ...)
    )
)]
struct ApiDoc;
```

### 3.6. File Modifications

```
src/server/
├── handlers.rs          [UPDATE] Add utoipa annotations
└── mod.rs              [UPDATE] Configure Swagger UI routes

Cargo.toml              [UPDATE] Add utoipa dependencies
```

### 3.7. Success Criteria

#### 3.7.1. Functional Requirements
- ✅ Swagger UI accessible at `http://localhost:3000/api/docs`
- ✅ OpenAPI spec downloadable at `/api/openapi.json`
- ✅ All 5 data endpoints documented
- ✅ Interactive "Try it out" works in Swagger UI
- ✅ Query parameters rendered with input fields
- ✅ Response examples visible
- ✅ Schema definitions complete

#### 3.7.2. Quality Requirements
- ✅ All parameters have descriptions
- ✅ Response schemas are accurate
- ✅ Examples are realistic (not placeholder data)
- ✅ API metadata (title, version) is present
- ✅ No compilation warnings from utoipa macros

### 3.8. Testing Approach

- Open Swagger UI in browser
- Test each endpoint via "Try it out"
- Verify query parameters work (filtering, pagination)
- Download OpenAPI JSON and validate structure
- Test with external tools (Postman, curl with spec import)

### 3.9. Future Extensions (Not in Task 3)

- API authentication (JWT, API keys)
- Rate limiting per endpoint
- API versioning strategy
- CORS configuration
- Request validation middleware
- Response compression
- Caching headers

### 3.10. Dependencies

This task depends on:
- ✅ Task 1: Parquet data available
- ✅ Task 2: REST API handlers exist

This task enables:
- 🔲 External integrations (data consumers can discover API)
- 🔲 Client library generation (from OpenAPI spec)
- 🔲 API testing automation (from spec)
