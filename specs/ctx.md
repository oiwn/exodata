### Current Task
Stellar Hosts Aggregation Example

### Goal
Create an example that demonstrates aggregation queries over the stellar hosts dataset, providing relevant statistics and insights.

### Stellar Host Dataset Information
- **46,887 stars** hosting exoplanets
- **136 columns** with stellar properties (temperature, mass, radius, metallicity, etc.)
- **Columns include:**
  - Basic: hostname, hd_name, hip_name, tic_id, gaia_ids
  - Position: ra, dec, rastr, decstr, glon, glat, elon, elat
  - Stellar Properties: st_teff, st_mass, st_rad, st_logg, st_lum, st_age, st_met, st_radv, st_vsin
  - Photometry: sy_vmag, sy_bmag, sy_jmag, sy_hmag, sy_kmag, sy_gmag, sy_gaiamag, sy_kepmag
  - Distance: sy_dist (when available)

### Aggregation Query Ideas

#### 1. **Stellar Property Distributions**
- Temperature distribution histogram (st_teff)
- Mass distribution (st_mass) 
- Radius distribution (st_rad)
- Metallicity distribution (st_met)
- Age distribution (st_age)

#### 2. **Discovery Statistics**
- Stars by discovery method of their planets
- Stars by discovery decade
- Stars by discovery facility
- Geographic distribution by sky coordinates

#### 3. **Multi-dimensional Analysis**
- Mass vs Temperature correlation
- Radius vs Temperature correlation
- Metallicity distribution across stellar types
- Age distribution in stellar population

#### 4. **Photometric Statistics**
- Magnitude distributions across different bands
- Color indices relationships
- Photometric completeness by band

#### 5. **Distance Analysis**
- Distance distribution of sample
- Volume of space surveyed
- Distance vs magnitude relationships

#### 6. **Cross-referenced Data**
- Match rate between different catalogs (HD, HIP, TIC, GAIA)
- Catalog completeness by stellar magnitude
- Position accuracy across catalogs

### Implementation Approach

#### Example Query: **Stellar Temperature Analysis**
```rust
// Load data
let df = load_data("data/stellarhosts.vot")?;

// Temperature distribution
let temp_stats = df
    .lazy()
    .select([
        col("st_teff"),
        // Create temperature bins
        (col("st_teff") / lit(1000) * lit(10)).cast(DataType::Int32).alias("temp_bin_k")
    ])
    .filter(col("st_teff").is_not_null())
    .collect()?;

// Get histogram data
let temp_histogram = temp_stats
    .group_by(["temp_bin_k"])
    .agg([
        count().alias("star_count"),
    ])
    .sort("temp_bin_k", SortOptions::default())
    .collect()?;
```

#### Example Query: **Discovery Timeline**
```rust
// Discovery timeline
let discovery_stats = df
    .lazy()
    .select([
        col("hostname"),
        // Extract decade from disc_year
        (col("disc_year") / lit(10) * lit(10)).cast(DataType::Int32).alias("discovery_decade")
    ])
    .filter(col("disc_year").is_not_null())
    .group_by(["discovery_decade"])
    .agg([
        count().alias("stars_discovered"),
        // Get median temperature for each decade
        col("st_teff").median().alias("median_temp")
    ])
    .sort("discovery_decade", SortOptions::default())
    .collect()?;
```

#### Example Query: **Catalog Cross-matching**
```rust
// Cross-matching between catalogs
let catalog_stats = df
    .lazy()
    .select([
        col("hostname"),
        // Count available identifiers for each star
        col("hd_name").is_not_null().cast(DataType::Int32).alias("has_hd"),
        col("hip_name").is_not_null().cast(DataType::Int32).alias("has_hip"),
        col("tic_id").is_not_null().cast(DataType::Int32).alias("has_tic"),
        col("gaia_dr2_id").is_not_null().cast(DataType::Int32).alias("has_gaia_dr2"),
        col("gaia_dr3_id").is_not_null().cast(DataType::Int32).alias("has_gaia_dr3"),
        // Count total identifiers available
        (col("hd_name").is_not_null() + col("hip_name").is_not_null() + 
         col("tic_id").is_not_null() + col("gaia_dr2_id").is_not_null() + 
         col("gaia_dr3_id").is_not_null()).cast(DataType::Int32).alias("total_ids")
    ])
    .filter(col("hostname").is_not_null())
    .group_by([all()])
    .agg([
        sum("has_hd").alias("stars_with_hd"),
        sum("has_hip").alias("stars_with_hip"), 
        sum("has_tic").alias("stars_with_tic"),
        sum("has_gaia_dr2").alias("stars_with_gaia_dr2"),
        sum("has_gaia_dr3").alias("stars_with_gaia_dr3"),
        sum("total_ids").alias("total_catalog_entries"),
        // Compute match rates
        (sum("has_hd") / lit(46887)).alias("hd_completion_rate"),
        (sum("has_gaia_dr2") / lit(46887)).alias("gaia_dr2_completion_rate")
    ])
    .collect()?;
```

### Output Format

Results should be displayed as formatted tables with:
- **Descriptive headers** for each aggregation
- **Summary statistics** (count, mean, median, std, min, max)
- **Histogram data** with appropriate binning
- **Cross-tabulation** for categorical relationships
- **Correlation coefficients** for continuous variables

### Performance Considerations

- Use lazy evaluation for complex aggregations
- Apply filters early to reduce dataset size
- Use appropriate data types to minimize memory
- Consider sampling for very large aggregations
- Cache intermediate results where beneficial

### Next Steps

1. Implement basic aggregation functions
2. Create formatted output utilities
3. Add visualization support (histograms, scatter plots)
4. Extend to exoplanets dataset aggregation
5. Create cross-dataset aggregation capabilities

---

## MY NOTES: Interactive Aggregation Terminal UI

### Concept
Create an interactive terminal interface using `ratatui` crate for stellar hosts aggregation analysis.

### UI Design

#### Main Interface Layout
```
┌ Stellar Hosts Aggregation Explorer ─────────────────────────────────────┐
│                                                        │
│ [F1] Temperature Distribution    [F2] Discovery Timeline    │
│ [F3] Catalog Cross-match    [F4] Photometric Stats   │
│                                                        │
│ Active Tab: Temperature Distribution                    │
└────────────────────────────────────────────────────────────────┘

┌ Data Results ────────────────────────────────────────────────┐
│                                                        │
│ Temperature Range    | Star Count    | Percentage    │
│ 3000-4000 K        | 1,234         | 2.6%         │
│ 4000-5000 K        | 15,678        | 33.4%        │
│ 5000-6000 K        | 18,456        | 39.4%        │
│ 6000-7000 K        | 8,923          | 19.0%        │
│ 7000-8000 K        | 2,596          | 5.5%         │
│                                                        │
│ Total Stars: 46,887  | Mean Temp: 5,234 K              │
└────────────────────────────────────────────────────────────────┘

┌ Controls ──────────────────────────────────────────────────────┐
│                                                        │
│ [↑↓] Navigate     | [Enter] Select    | [q] Quit      │
│ [←→] Switch Tab  | [r] Refresh      | [s] Save       │
└────────────────────────────────────────────────────────────────┘
```

#### Tab System

**F1: Temperature Distribution**
- Histogram of stellar temperatures (st_teff)
- Temperature bins: 3000-4000K, 4000-5000K, etc.
- Statistics: mean, median, std dev, quartiles
- Bar chart visualization using ratatui block characters

**F2: Discovery Timeline**
- Stars discovered per decade
- Discovery methods breakdown (Transit, Radial Velocity, etc.)
- Discovery facilities ranking
- Timeline visualization

**F3: Catalog Cross-match**
- Cross-reference between HD, HIP, TIC, GAIA catalogs
- Match rates and completeness statistics
- Venn diagram-style visualization

**F4: Photometric Statistics**
- Magnitude distributions across photometric bands
- Color indices relationships
- Band completeness analysis
- Multi-band comparison charts

### Implementation Plan

#### Phase 1: Basic Tab Framework
```rust
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Tabs},
    Terminal,
};

struct App {
    current_tab: usize,
    temperature_data: Vec<TemperatureBin>,
    discovery_data: Vec<DecadeData>,
    catalog_data: CatalogStats,
    photometric_data: PhotometricStats,
    // Status and loading
    loading: bool,
    status_message: String,
}

enum Tab {
    Temperature,
    Discovery,
    Catalog,
    Photometric,
}
```

#### Phase 2: Data Processing Backend
```rust
// Lazy evaluation for aggregations
fn compute_temperature_distribution(df: &DataFrame) -> Vec<TemperatureBin> {
    df.lazy()
        .filter(col("st_teff").is_not_null())
        .select([
            col("st_teff"),
            // Create temperature bins
            ((col("st_teff") - lit(3000)) / lit(1000))
                .clip(0, 9)
                .cast(DataType::Int32)
                .alias("temp_bin")
        ])
        .group_by(["temp_bin"])
        .agg([
            count().alias("star_count"),
            col("st_teff").mean().alias("mean_temp"),
            col("st_teff").median().alias("median_temp"),
        ])
        .sort("temp_bin", SortOptions::default())
        .collect()
        .unwrap()
}

fn compute_discovery_timeline(df: &DataFrame) -> Vec<DecadeData> {
    df.lazy()
        .filter(col("disc_year").is_not_null())
        .select([
            col("disc_year"),
            // Extract decade
            (col("disc_year") / lit(10) * lit(10))
                .cast(DataType::Int32)
                .alias("decade"),
            col("discoverymethod"),
            col("hostname"),
        ])
        .group_by(["decade"])
        .agg([
            count().alias("stars_discovered"),
            // Count by discovery method
            col("discoverymethod").n_unique().alias("methods_used"),
        ])
        .sort("decade", SortOptions::default())
        .collect()
        .unwrap()
}
```

#### Phase 3: UI Components

**Temperature Tab**
```rust
fn render_temperature_tab(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ])
        .split(area);

    // Summary stats
    let summary_text = format!(
        "Total Stars: {} | Mean: {}K | Median: {}K",
        app.temperature_data.iter().map(|b| b.count).sum::<usize>(),
        format!("{:.0}", compute_temp_mean(&app.temperature_data)),
        format!("{:.0}", compute_temp_median(&app.temperature_data))
    );

    let summary = Paragraph::new(summary_text)
        .block(Block::default().borders(Borders::ALL).title("Temperature Summary"));
    f.render_widget(summary, chunks[0]);

    // Histogram
    let max_count = app.temperature_data.iter().map(|b| b.count).max().unwrap_or(0);
    let histogram_items: Vec<ListItem> = app.temperature_data
        .iter()
        .enumerate()
        .map(|(i, bin)| {
            let bar_width = (bin.count * 30 / max_count) as u16;
            let bar = "█".repeat(bar_width as usize);
            let bar_space = " ".repeat(30 - bar_width as usize);
            
            ListItem::new(format!(
                "{}K  | {} {} | {:.1}%",
                3000 + i * 1000,
                bar,
                bar_space,
                bin.count,
                bin.count as f64 / 46887.0 * 100.0
            ))
        })
        .collect();

    let histogram = List::new(histogram_items)
        .block(Block::default().borders(Borders::ALL).title("Temperature Distribution"));
    f.render_widget(histogram, chunks[1]);
}
```

#### Phase 4: Interaction Controls

**Keyboard Navigation**
- `F1-F4`: Switch between tabs
- `↑↓`: Navigate within lists
- `←→`: Navigate between panels
- `r`: Refresh/recompute data
- `s`: Save current view to file
- `q`: Quit application

**Data Refresh**
- Background computation for large aggregations
- Progress indicators during processing
- Caching of computed results

### Performance Optimizations

1. **Lazy Evaluation**: Use Polars lazy for all aggregations
2. **Progressive Loading**: Compute data in chunks for UI updates
3. **Caching**: Store computed aggregations to avoid recomputation
4. **Memory Management**: Use streaming for very large datasets

### File Output Options

```
Export Format Options:
[1] CSV - Tabular data
[2] JSON - Structured data  
[3] TXT - Formatted report
[4] SVG - Charts and graphs
```

### Next Implementation Steps

1. ✅ Add `ratatui` dependency to Cargo.toml
2. 🔄 Create basic app structure and tab system
3. 🔄 Implement temperature distribution aggregation and rendering
4. 🔄 Add discovery timeline functionality
5. 🔄 Implement catalog cross-matching visualization
6. 🔄 Add photometric statistics display
7. 🔄 Add keyboard navigation and controls
8. 🔄 Add export/save functionality
9. 🔄 Add error handling and data validation
10. 🔄 Performance testing with full dataset

### Dependencies Required
```toml
[dependencies]
ratatui = "0.28"
crossterm = "0.28"
# Add to existing dependencies
```

This approach provides an intuitive, interactive way to explore stellar hosts aggregation data with visual feedback and multiple analysis perspectives.