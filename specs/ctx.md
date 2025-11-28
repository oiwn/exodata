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


# ^^^ MY NOTES

I think, it's decent plan, let's do example using "ratatui" crate, i already added into the "Cargo.toml". I think there are appropirate widgets for my cases. Bars, histograms etc. I see it as screen which display the data and i can switch between tabs (tab per aggregation data query) using arrow keys. Let's highlight it as spec.
