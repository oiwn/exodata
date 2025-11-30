### Current Task
Data Performance Optimization and Interactive Exploration

### Goal
Create fast-loading data formats and interactive tools for exploring exoplanet and stellar hosts datasets.

### Performance Issues Identified
- **VOTable Loading**: ~8 seconds for 46,887 rows (stellarhosts)
- **Missing Discovery Data**: Stellar hosts dataset contains no discovery information
- **High Missing Values**: 19-83% missing in key stellar properties
- **Memory Inefficiency**: 1.28 KB per row due to VOTable parsing overhead

#### Root Cause Analysis (^^^ Need to figure out why)
**VOTable Loading Slowness:**
- **XML Parsing Overhead**: VOTable uses verbose XML format with lots of metadata
- **Memory Allocations**: Each cell value goes through multiple conversion steps (XML → string → type conversion)
- **Schema Discovery**: Needs to parse entire file structure first, then data
- **Row-by-Row Processing**: Can't leverage vectorized operations during loading

**Evidence from benchmark:**
- 46,887 rows = ~8 seconds loading
- 1.28 KB per row memory usage (very inefficient)
- Even 100 row subsets take 8+ seconds (shows parsing overhead dominates)

### Immediate Solutions Needed

#### 1. CLI Format Conversion Tools
Convert VOTable files to high-performance formats for 10-50x faster loading:
```bash
# Convert all .vot files in data/ directory to .parquet
cargo run -- convert-raw-files
```

^^^ Only parquet, same folder, batch conversion of all vot files
*Status:* Implemented (`convert-raw-files`). Validates row/col counts after conversion. Current timings on Apple Silicon: exoplanets.vot ~20.5s (17.5s metadata parse + 0.37s rows + 2.7s write/validate); stellarhosts.vot ~9.3s (7.8s metadata parse + 0.15s rows + 1.3s write/validate). Compression ~22x (375.5MB → 16.2MB, 158.8MB → 7.7MB).

#### 2. Data Inspection Examples
Create focused inspection tools for understanding dataset structure:
```bash
# Stellar hosts inspection (catalog cross-match, photometric bands, stellar properties)
cargo run --example stellarhosts_inspection

# Exoplanets inspection (discovery timeline, orbital characteristics, physical properties)
cargo run --example exoplanets_inspection
```

^^^ Holy shit, why benchmark is there?

#### 3. Performance Benchmark
```bash
cargo run --example performance_benchmark
```

**Benchmark Requirements:**
- Show parquet loading times (current impl); VOT parsing remains dominated by metadata (see timings above)
- Compare file sizes and memory usage
- Demonstrate speedup factor when VOT timing is added

^^^ That's correct

#### 4. Interactive TUI Explorer
Build terminal-based interactive exploration tool:
```bash
# Separate examples for each dataset
cargo run --example stellarhosts_tui
cargo run --example exoplanets_tui
```

**Key TUI Organization Questions:**
- What aggregations are most valuable to show data properties?
- Should we combine data or keep separate tabs?
- What export formats do users need?
- How to represent large datasets in limited terminal space?

### Key Insights from Data Inspection

#### Stellar Hosts Dataset (46,887 stars, 136 columns)
- **Catalog Coverage**: HD (14.5%), HIP (15.2%), TIC (98.0%), GAIA (96-97%)
- **Stellar Properties**: Only 28,662/46,887 stars have mass data (61.2%)
- **Photometric Data**: Complete coverage across 8 bands (V, B, J, H, K, G, Gaia, Kepler)
- **Missing Discovery Info**: This is host star data, not discovery data

#### Temperature Distribution Analysis
- **Peak**: 45.9% of stars have 5000-6000K (G-type stars)
- **Distribution**: 5.8% M-type (3000-4000K), 12.9% K-type (4000-5000K)
- **Hot Stars**: Only 0.9% of stars >7000K (B/A-type)

#### Planet Count Analysis
- **Multiple Planet Systems**: Many hosts have 2+ planets
- **Data Quality**: Need to analyze distribution from sy_pnum column

### Implementation Plan

#### Phase 1: Format Conversion (Immediate - 1-2 days)
- [ ] Add parquet feature to Cargo.toml
- [ ] Create CLI conversion tool with progress bars
- [ ] Implement validation and integrity checks
- [ ] Benchmark loading performance improvements

#### Phase 2: Data Inspection (1 day)
- [ ] Create unified inspection framework
- [ ] Implement specific stellar hosts analysis
- [ ] Implement exoplanets discovery analysis  
- [ ] Add comprehensive statistical summaries

#### Phase 3: Interactive TUI (2-3 days)
- [ ] Set up ratatui framework and basic layout
- [ ] Implement data table browser with pagination
- [ ] Add interactive filtering and statistics panels
- [ ] Create export functionality for filtered data

### Performance Targets
- **Loading Time**: <1 second from optimized formats
- **Memory Usage**: Maintain <200MB total for both datasets
- **Interactive Response**: <100ms for filtering/aggregation
- **Export Speed**: <2 seconds for 10,000 row datasets

### File Structure Reorganization
```
data/
├── stellarhosts.vot      # 158.8MB, slow loading
├── exoplanets.vot        # 375.5MB, slow loading
├── stellarhosts.parquet  # 7.7MB, ~80ms load
├── exoplanets.parquet    # 16.2MB, ~160ms load
```

### Key Takeaways
1. **VOTable is a bottleneck** - convert to columnar formats immediately
2. **Stellar hosts = catalog data** - discovery analysis needs exoplanets dataset
3. **Data quality varies** - different columns have very different missing rates
4. **Aggregation potential** - rich dataset for stellar population analysis

---

# TODO

## Phase 1: Format Conversion (Priority 1)
- [x] Add parquet feature to Cargo.toml
- [x] Create `convert-raw-files` CLI command
  - [x] Auto-discover all .vot files in data/ directory
  - [x] Convert to parquet format with progress bars (metadata parse still dominant)
  - [x] Validate output integrity (row/col parity)
  - [x] Show before/after performance comparison (VOT vs parquet timing still todo)
- [x] Run conversion on stellarhosts.vot and exoplanets.vot
- [x] Update all existing examples to use .parquet files (parquet-only loaders)

## Phase 2: Data Inspection (Priority 2)
- [ ] Create `stellarhosts_inspection` example
  - [ ] Column search and pattern matching
  - [ ] Catalog cross-match analysis
  - [ ] Photometric band statistics
  - [ ] Stellar property distribution analysis
- [ ] Create `exoplanets_inspection` example
  - [ ] Discovery timeline analysis (exoplanets dataset has disc_year)
  - [ ] Discovery method breakdown
  - [ ] Orbital characteristic analysis
  - [ ] Physical property distribution
- [ ] Update `performance_benchmark` example
  - [ ] Compare VOTable vs parquet loading times (parquet-only timing done; add VOT timing later)
  - [ ] Show memory usage differences
  - [ ] Show file size reduction
  - [ ] Demonstrate query performance differences

## Phase 3: Interactive TUI (Priority 3)
- [ ] Answer TUI organization questions:
  - [ ] What aggregations are most valuable for scientists?
  - [ ] How to handle datasets with different columns?
  - [ ] Should we combine data or keep separate tabs?
  - [ ] What export formats do users need?
  - [ ] How to represent large datasets in terminal?
- [ ] Create `stellarhosts_tui` example
- [ ] Create `exoplanets_tui` example
  - [ ] Basic framework with ratatui
  - [ ] Data table browser with pagination
  - [ ] Interactive filtering system
  - [ ] Statistics panels with aggregations
  - [ ] Export functionality

## Phase 4: Integration & Polish (Priority 4)
- [ ] Update web application to use parquet files
- [ ] Add caching for query results
- [ ] Error handling and data validation
- [ ] Documentation and usage examples
- [ ] Performance optimization and testing

## Technical Debts & Questions
- [ ] Should we cache converted files in memory for repeated access?
- [ ] How to handle dataset updates (re-conversion needed)?
- [ ] Should we add CSV export option for compatibility?
- [ ] Memory management for large aggregations in TUI?
- [ ] What should be the default data format for new users?
- [ ] Check all fields are in format suited for calculations
- [ ] Validate data types are appropriate for aggregations
