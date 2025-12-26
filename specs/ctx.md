# Current Context

**Goal**: Add descriptive tooltips to table column headers using official NASA Exoplanet Archive descriptions.

**See**: `specs/column-metadata.md` for detailed implementation plan

**Status**: Table component updated to support tooltips, need to implement metadata in exo-core

**Tasks**:
1. ✅ Update Table component to accept `column_descriptions` prop
2. ✅ Research NASA Exoplanet Archive column documentation
3. ⏳ Create `exo-core/src/metadata.rs` module
4. ⏳ Implement column metadata for planets and stellar hosts
5. ⏳ Integrate metadata into frontend tables

**Potential future enhancements:**
- Add search/filter functionality to tables
- Add detailed view pages for individual planets/stars
- Add data visualizations (charts, graphs)
- Add export functionality (CSV, JSON)
- Add column visibility toggles
- Add more statistical insights to overview page
