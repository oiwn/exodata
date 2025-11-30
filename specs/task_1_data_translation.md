## 1. Data Handling

This document outlines strategy for parsing, storing, and querying exoplanet and stellar host data. The primary goal is to load data into high-performance structures that support complex analytical queries.

### 1.1. Data Sources
The exoplanet and stellar host data are provided as VOTable files, acquired from NASA Exoplanet Archive via Just commands:
- `data/stellarhosts.vot` (167MB) - Information about stars hosting exoplanets
- `data/exoplanets.vot` (394MB) - Confirmed exoplanets data from `ps` table

### 1.2. Core Technology
We use two main Rust crates for this task:
- **`votable`**: To parse raw VOTable files
- **`polars`**: To store data in `DataFrame` and perform all subsequent querying and analysis

### 1.3. Performance Issues & Solutions

#### 1.3.1. Current Problems
- **Slow Loading**: VOTable parsing takes ~8 seconds for 46,887 rows
- **No Discovery Data**: Stellar hosts dataset lacks discovery information
- **High Missing Values**: 19-83% missing values in key stellar properties

#### 1.3.2. Proposed Solutions

**CLI Tool for Fast Format Conversion**
```bash
# Convert VOTable to high-performance formats
cargo run -- convert-to-parquet data/stellarhosts.vot
cargo run -- convert-to-parquet data/exoplanets.vot
```

**Supported Formats**
- **Parquet**: 10-50x faster loading, smaller file size
- **CSV**: Universal compatibility, easy inspection
- **Feather/IPC**: Fastest Polars native format

### 1.4. Data Exploration Tools

#### 1.4.1. Data Inspection Examples
```bash
# Quick inspection of datasets
cargo run --example data_inspection
cargo run --example exoplanets_inspection

# Performance benchmarking
cargo run --example data_loading_benchmark
```

#### 1.4.2. Interactive TUI Exploration
```bash
# Terminal UI for interactive exploration
cargo run --example tui_explorer
```

### 1.5. CLI Tool Specification

#### 1.5.1. Data Format Conversion Tool
```bash
# Convert VOTable to fast-loading formats
cargo run -- convert --input data/stellarhosts.vot --output data/stellarhosts.parquet --format parquet
cargo run -- convert --input data/exoplanets.vot --output data/exoplanets.parquet --format parquet

# Batch conversion
cargo run -- convert-all --input-dir data/ --output-dir data/converted/
```

**Features**
- Progress bars for large files
- Memory usage monitoring
- Format-specific optimizations
- Validation of output integrity

#### 1.5.2. Data Inspection Tool
```bash
# Stellar hosts inspection
cargo run -- inspect stellarhosts --limit 1000 --columns hostname,st_teff,st_mass

# Exoplanets inspection  
cargo run -- inspect exoplanets --limit 1000 --columns pl_name,pl_orbper,pl_bmasse

# Column search
cargo run -- inspect stellarhosts --search discovery
cargo run -- inspect exoplanets --search disc_year
```

**Features**
- Column pattern matching
- Data type information
- Missing value analysis
- Statistical summaries
- Sample data preview

#### 1.5.3. Interactive TUI Explorer
```bash
# Launch interactive TUI explorer
cargo run -- tui-explorer --file data/stellarhosts.parquet
cargo run -- tui-explorer --file data/exoplanets.parquet
```

**TUI Features**
- **Data Browser**: Paginated view of all rows/columns
- **Filter Panel**: Interactive filtering by column values
- **Statistics View**: Real-time aggregation statistics
- **Export Options**: CSV, JSON, filtered data extraction

### 1.6. Implementation Tasks

#### Phase 1: CLI Format Conversion
- [ ] Add parquet/feather support to dependencies
- [ ] Implement format conversion with progress bars
- [ ] Add validation for converted files
- [ ] Benchmark performance improvements

#### Phase 2: Data Inspection Tools
- [ ] Create unified inspection example for both datasets
- [ ] Add column search and pattern matching
- [ ] Implement statistical analysis for key columns
- [ ] Add data quality assessment

#### Phase 3: Interactive TUI Explorer
- [ ] Create TUI framework with `ratatui`
- [ ] Implement data table with pagination
- [ ] Add interactive filtering system
- [ ] Create statistics panel with aggregations
- [ ] Add export functionality

### 1.7. Performance Targets
- **Loading Time**: <1 second for full datasets (from optimized formats)
- **Memory Usage**: Maintain <200MB total for both datasets
- **Query Response**: <100ms for common aggregations
- **Export Speed**: <2 seconds for 10,000 row exports

### 1.8. File Management Strategy
```
data/
├── raw/                    # Original VOTable files
│   ├── stellarhosts.vot
│   └── exoplanets.vot
├── processed/              # Converted fast-loading formats
│   ├── stellarhosts.parquet
│   ├── stellarhosts.ipc
│   ├── exoplanets.parquet
│   └── exoplanets.ipc
└── cache/                 # Query result caches
    └── *.cache
```

### 1.9. Next Steps
1. **Implement CLI conversion tools** for immediate performance gains
2. **Create data inspection examples** for better understanding of datasets
3. **Build TUI explorer** for interactive analysis
4. **Integrate with web application** for fast backend data loading