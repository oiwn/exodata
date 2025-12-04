### Current Task
Web Application Integration

**Specification File**: `specs/task_2_web_integration.md`

### Goal
Integrate high-performance parquet data loading into the Leptos web application to provide fast, interactive browsing of exoplanet and stellar hosts datasets.

### Current State
- **Backend**: Uses Axum with VOTable loading (slow)
- **Frontend**: Leptos-based web interface
- **Data Format**: Currently loads VOTable files directly
- **Performance**: Web pages load slowly due to VOTable parsing overhead

### Integration Requirements
1. **Backend Updates**
   - Replace VOTable loading with parquet loading
   - Use existing `load_parquet` function from tables::common
   - Update API endpoints to handle parquet-based queries

2. **Frontend Updates**
   - Update data fetching to use new parquet-based endpoints
   - Improve loading indicators for better UX
   - Add progressive loading for large datasets
   - Implement search functionality

3. **Performance Targets**
   - Page load time: <2 seconds (vs current 8+ seconds)
   - Search response: <500ms
   - Filter application: <1 second
   - Data export: <5 seconds for 10,000 rows

### Technical Approach

#### Backend Integration
- Modify data loading functions in `src/app.rs` or `src/main.rs`
- Update server endpoints to use parquet loader
- Implement query optimization for common requests
- Add error handling for missing parquet files

#### Frontend Integration
- Update component data fetching logic
- Add loading states and error handling
- Implement pagination for large datasets
- Add real-time search capabilities

### API Design
```
GET /api/exoplanets                    # Load exoplanets data (parquet)
GET /api/stellarhosts                 # Load stellar hosts data (parquet)
GET /api/search?query=...            # SQL query across datasets
GET /api/export?format=...           # Export filtered data
GET /api/system/{hostname}           # Get specific solar system data
GET /api/planet/{pl_name}           # Get specific planet data
```

**SQL Query Support**: Leverage Polars SQL execution context for `/api/search?query=...` endpoint to enable complex filtering and aggregations.

### File Structure Updates
```
src/
├── main.rs                 # Update with parquet loading
├── app.rs                  # Update data fetching logic and routing
├── server/
│   ├── handlers.rs        # Add parquet-based endpoints
│   └── api.rs           # SQL query execution for search
└── components/
    ├── data_table.rs       # Add pagination/loading states
    ├── search.rs         # Add search functionality
    ├── system_page.rs     # Solar system detail page
    └── planet_page.rs    # Planet detail page
```

**Individual System Pages**: Add routing for `/system/{hostname}` and `/planet/{pl_name}` pages with detailed information about specific solar systems and individual planets.

# TODO

## Web Application Integration (Priority 1)

### Phase 1: Backend Parquet Integration (Immediate)
- [ ] Identify current VOTable loading locations in backend
- [ ] Replace with `load_parquet` function calls
- [ ] Update error handling for parquet file dependencies
- [ ] Test backend performance improvements
- [ ] Ensure API endpoints return same data structure

### Phase 2: Frontend Data Fetching Updates
- [ ] Locate data fetching logic in Leptos components
- [ ] Update to handle faster parquet-based responses
- [ ] Add loading indicators and error states
- [ ] Test frontend with new backend performance
- [ ] Ensure UI responsiveness during data loading

### Phase 3: Enhanced User Experience
- [ ] Add real-time SQL-based search functionality
- [ ] Implement pagination for large datasets
- [ ] Add data export capabilities (CSV, JSON)
- [ ] Create solar system detail pages
- [ ] Create planet detail pages
- [ ] Optimize loading states and user feedback
- [ ] Test cross-browser compatibility

### Phase 4: Advanced Features (Future)
- [ ] Add data visualization components
- [ ] Implement advanced filtering options
- [ ] Add bookmarking/favorites functionality
- [ ] Optimize for mobile devices
- [ ] Add offline data caching

### Technical Questions to Resolve
- [ ] How should the app handle missing parquet files?
- [ ] What's the best caching strategy for web requests?
- [ ] Should we implement server-side rendering with parquet data?
- [ ] How to handle concurrent users accessing same datasets?
- [ ] How should SQL queries be validated and secured?
- [ ] What's the optimal data structure for individual system pages?
