# Data Layer Specification: exo-core

This document describes the `exo-core` library, which provides all data processing functionality for the exoplanets catalog.

## Overview

`exo-core` is a pure Rust library that handles:
- VOTable XML parsing and analysis
- Parquet file I/O operations
- DataFrame operations using Polars
- Statistical aggregations and analysis
- Data conversions between formats

This library has **no CLI or web dependencies** and can be used by any Rust application.

## Module Structure

```
crates/exo-core/src/
├── lib.rs              # Library root, re-exports modules
├── common.rs           # VOTable utilities and struct generation
└── tables/
    ├── mod.rs          # Table module declarations
    ├── common.rs       # Parquet I/O and basic stats
    ├── aggregation.rs  # Statistical aggregations
    ├── votable_loader.rs  # VOTable parsing with progress
    ├── conversion.rs   # VOTable to Parquet conversion
    ├── exoplanets.rs   # Exoplanet-specific data loading
    └── stellarhosts.rs # Stellar host-specific data loading
```

## Dependencies

```toml
polars = { version = "0.52.0", features = ["lazy", "temporal", "timezones", "parquet"] }
votable = "0.6.3"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0.100"
indicatif = "0.18.3"
```

## Core Modules

### 1. `common.rs` (Root Level)

VOTable analysis and code generation utilities.

**Key Functions:**
- `print_votable_headers(path: &str)` - Display VOTable field definitions
- `detect_nullable_columns(path: &str) -> HashSet<usize>` - Scan for nullable fields
- `structure_from_votables_codegen(path: &str, name: &str)` - Generate Rust struct from VOTable schema
- `extract_columns_types(path: &str)` - Analyze column types

**Usage:**
```rust
use exo_core::common::print_votable_headers;

print_votable_headers("data/stellarhosts.vot");
```

### 2. `tables::common`

Parquet I/O and basic DataFrame operations.

**Key Functions:**
- `load_parquet(path: &str, limit: Option<usize>) -> Result<DataFrame>`
- `count_non_null_values(df: &DataFrame, col_name: &str) -> Result<usize>`
- `get_numeric_stats(df: &DataFrame, col_name: &str) -> Result<Option<NumericStats>>`
- `create_histogram(data: Vec<f64>, bins: usize) -> Vec<(f64, f64, usize)>`
- `print_histogram(histogram: Vec<(f64, f64, usize)>, title: &str)`

**Data Structures:**
```rust
pub struct NumericStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std: f64,
    pub min: f64,
    pub max: f64,
}
```

**Usage:**
```rust
use exo_core::tables::common::load_parquet;

let df = load_parquet("data/exoplanets.parquet", Some(100))?;
println!("Loaded {} rows", df.height());
```

### 3. `tables::aggregation`

Statistical analysis and aggregation functions.

**Key Data Structures:**
```rust
pub struct TemperatureBin {
    pub range: String,
    pub min_temp: f64,
    pub max_temp: f64,
    pub star_count: u32,
    pub percentage: f64,
}

pub struct DecadeData {
    pub decade: i32,
    pub stars_discovered: u32,
    pub discovery_methods: HashMap<String, u32>,
    pub median_temp: Option<f64>,
}

pub struct CatalogStats {
    pub total_stars: u32,
    pub hd_match_rate: f64,
    pub hip_match_rate: f64,
    pub tic_match_rate: f64,
    pub gaia_dr2_match_rate: f64,
    pub gaia_dr3_match_rate: f64,
}

pub struct PhotometricStats {
    pub band_stats: HashMap<String, BandStats>,
    pub color_indices: HashMap<String, f64>,
}
```

**Key Functions:**
- `temperature_distribution(df: &DataFrame) -> Result<Vec<TemperatureBin>>`
- `discovery_timeline(df: &DataFrame) -> Result<Vec<DecadeData>>`
- `catalog_cross_match_stats(df: &DataFrame) -> Result<CatalogStats>`
- `photometric_analysis(df: &DataFrame) -> Result<PhotometricStats>`
- `get_total_counts(stellarhosts_df: &DataFrame, exoplanets_df: &DataFrame) -> (usize, usize)`
- `get_avg_temperature(df: &DataFrame) -> Option<f64>`
- `get_avg_distance(df: &DataFrame) -> Option<f64>`
- `get_discovery_methods(df: &DataFrame, limit: usize) -> Vec<(String, usize)>`
- `get_planet_size_categories(df: &DataFrame) -> Vec<(String, usize)>`

**Usage:**
```rust
use exo_core::tables::aggregation::temperature_distribution;

let bins = temperature_distribution(&df)?;
for bin in bins {
    println!("{}: {} stars ({:.1}%)",
        bin.range, bin.star_count, bin.percentage);
}
```

### 4. `tables::votable_loader`

VOTable XML parsing with progress tracking.

**Key Functions:**
- `load_votable_with_progress_timed(path: &str, progress: Option<&ProgressBar>) -> Result<DataFrame>`
- `load_votable_to_dataframe(path: &str) -> Result<DataFrame>`

**Features:**
- Streams large VOTable files efficiently
- Reports parsing progress with indicatif
- Converts VOTable data to Polars DataFrame
- Handles nullable fields appropriately

**Usage:**
```rust
use exo_core::tables::votable_loader::load_votable_to_dataframe;

let df = load_votable_to_dataframe("data/stellarhosts.vot")?;
```

### 5. `tables::conversion`

Convert VOTable files to Parquet format.

**Key Functions:**
- `convert_raw_files(data_dir: &Path) -> Result<()>`
- `convert_votable_to_parquet(vot_path: &Path, parquet_path: &Path) -> Result<()>`

**Features:**
- Batch conversion of all `.vot` files in a directory
- Progress bars for each file
- Automatic output naming (`.vot` → `.parquet`)
- Error handling with detailed messages

**Usage:**
```rust
use exo_core::tables::conversion::convert_raw_files;
use std::path::Path;

convert_raw_files(Path::new("data"))?;
```

### 6. `tables::exoplanets` & `tables::stellarhosts`

Dataset-specific data loading with column selection.

**Key Functions:**
- `load_data_with_limit(path: &str, limit: Option<usize>) -> Result<DataFrame>`
- `load_data(path: &str) -> Result<DataFrame>`

**Features:**
- Predefined column categories (basic, discovery, orbital, physical)
- Efficient loading with row limits
- Dataset-specific default columns

## Column Categories

### Stellar Hosts
- **Basic**: hostname, hd_name, hip_name, tic_id
- **Position**: ra, dec, rastr, decstr, glon, glat
- **Stellar Properties**: st_teff, st_mass, st_rad, st_logg, st_lum, st_age, st_met
- **Photometry**: sy_vmag, sy_bmag, sy_jmag, sy_hmag, sy_kmag, sy_gmag, sy_gaiamag

### Exoplanets
- **Basic**: pl_name, hostname, discoverymethod, disc_year
- **Discovery**: discoverymethod, disc_year, disc_facility
- **Orbital**: pl_orbper, pl_orbsmax, pl_orbeccen, pl_orbincl
- **Physical**: pl_rade, pl_bmasse, pl_dens, pl_eqt

## Usage Examples

### Load and analyze data
```rust
use exo_core::tables::common::load_parquet;
use exo_core::tables::aggregation::get_avg_temperature;

// Load data
let df = load_parquet("data/stellarhosts.parquet", None)?;

// Get statistics
let avg_temp = get_avg_temperature(&df);
println!("Average temperature: {:.2}K", avg_temp.unwrap_or(0.0));
```

### Convert VOTable to Parquet
```rust
use exo_core::tables::conversion::convert_raw_files;
use std::path::Path;

// Convert all .vot files in data/ directory
convert_raw_files(Path::new("data"))?;
```

### Generate aggregated statistics
```rust
use exo_core::tables::aggregation::{
    get_total_counts,
    get_discovery_methods,
    get_planet_size_categories
};

let (stars_count, planets_count) = get_total_counts(&stars_df, &planets_df);
let methods = get_discovery_methods(&planets_df, 10);
let sizes = get_planet_size_categories(&planets_df);
```

## Error Handling

All public functions return `Result<T, E>` where `E` is typically:
- `anyhow::Error` - For general errors
- `polars::error::PolarsError` - For DataFrame operations

**Best Practices:**
```rust
match load_parquet("data/file.parquet", None) {
    Ok(df) => {
        // Process dataframe
    }
    Err(e) => {
        eprintln!("Failed to load data: {}", e);
        return;
    }
}
```

## Performance Considerations

1. **Lazy Loading**: Use `limit` parameter when you don't need all rows
2. **Memory**: DataFrames are stored in-memory, consider row limits for large files
3. **Progress Bars**: Use for long-running operations to improve UX
4. **Arc<DataFrame>**: Use for sharing data across threads without cloning

## Testing

Run tests for exo-core:
```bash
cargo test --package exo-core
```

## Future Enhancements

Potential additions to the data layer:
- Diff calculations between datasets
- Data validation functions
- Export to other formats (CSV, JSON)
- Streaming aggregations for large files
- Custom column transformations
- Advanced filtering DSL
