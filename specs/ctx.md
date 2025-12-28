# Current Context

**Goal**: Add descriptive tooltips to table column headers using official NASA Exoplanet Archive descriptions.

**See**: `specs/column-metadata.md` for detailed implementation plan

**Status**: Table component updated to support tooltips, need to implement metadata in exo-core

**Progress**:
1. ✅ Update Table component to accept `column_descriptions` prop
2. ✅ Research NASA Exoplanet Archive column documentation
3. ✅ **COMPLETED**: Created `exo-core/src/metadata.rs` module
4. ✅ **COMPLETED**: Implemented VOTable metadata parser (extracts from .vot files)
5. ✅ **COMPLETED**: Added `view-metadata` CLI command to exo-cli
6. ⏳ **NEXT**: Integrate metadata into frontend tables

**What's Working**:
- ✅ `exo-core/src/metadata.rs` parses VOTable files and extracts column metadata
- ✅ `exo-cli view-metadata` command prints metadata to console
- ✅ Metadata includes: name, description, unit, datatype
- ✅ Source: Official NASA Exoplanet Archive VOTable files

**Potential future enhancements:**
- Add search/filter functionality to tables
- Add detailed view pages for individual planets/stars
- Add data visualizations (charts, graphs)
- Add export functionality (CSV, JSON)
- Add column visibility toggles
- Add more statistical insights to overview page
