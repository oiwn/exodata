// Overview aggregation functions for stellar hosts and exoplanets data analysis.
// These functions are used by the overview page and other parts of the application.

use polars::prelude::*;
use std::collections::HashMap;

/// Temperature distribution data for histogram visualization
#[derive(Debug, Clone)]
pub struct TemperatureBin {
    pub range: String,
    pub min_temp: f64,
    pub max_temp: f64,
    pub star_count: u32,
    pub percentage: f64,
}

/// Discovery timeline data organized by decade
#[derive(Debug, Clone)]
pub struct DecadeData {
    pub decade: i32,
    pub stars_discovered: u32,
    pub discovery_methods: HashMap<String, u32>,
    pub median_temp: Option<f64>,
}

/// Catalog cross-match statistics
#[derive(Debug, Clone)]
pub struct CatalogStats {
    pub total_stars: u32,
    pub hd_match_rate: f64,
    pub hip_match_rate: f64,
    pub tic_match_rate: f64,
    pub gaia_dr2_match_rate: f64,
    pub gaia_dr3_match_rate: f64,
    pub cross_match_matrix: Vec<Vec<u32>>, // Venn diagram data
}

/// Photometric statistics for multiple bands
#[derive(Debug, Clone)]
pub struct PhotometricStats {
    pub band_stats: HashMap<String, BandStats>,
    pub color_indices: HashMap<String, f64>,
}

/// Statistics for a specific photometric band
#[derive(Debug, Clone)]
pub struct BandStats {
    pub band_name: String,
    pub count: u32,
    pub mean_mag: f64,
    pub median_mag: f64,
    pub std_mag: f64,
    pub min_mag: f64,
    pub max_mag: f64,
}

/// Compute temperature distribution histogram
/// Creates bins from 3000K to 10000K in 1000K increments
pub fn temperature_distribution(df: &DataFrame) -> Vec<TemperatureBin> {
    if let Ok(st_teff_col) = df.column("st_teff") {
        if let Some(st_teff_series) = st_teff_col.as_series() {
            if let Ok(st_teff_data) = st_teff_series.f64() {
                let total_stars = st_teff_data.len() as f64;
                let mut bin_counts = vec![0; 7]; // 7 bins from 3000-10000K

                // Count values in each bin
                for opt_temp in st_teff_data.into_iter() {
                    if let Some(temp) = opt_temp {
                        if temp >= 3000.0 && temp <= 10000.0 {
                            let bin_index = ((temp - 3000.0) / 1000.0) as usize;
                            if bin_index < 7 {
                                bin_counts[bin_index] += 1;
                            }
                        }
                    }
                }

                // Create bins
                let mut bins = Vec::new();
                for (i, &count) in bin_counts.iter().enumerate() {
                    let min_temp = 3000.0 + (i as f64) * 1000.0;
                    let max_temp = min_temp + 1000.0;

                    bins.push(TemperatureBin {
                        range: format!("{:.0}-{:.0}K", min_temp, max_temp),
                        min_temp,
                        max_temp,
                        star_count: count as u32,
                        percentage: (count as f64 / total_stars) * 100.0,
                    });
                }

                return bins;
            }
        }
    }

    vec![]
}

/// Compute discovery timeline by decade
/// Analyzes when stars were discovered and their properties
pub fn discovery_timeline(df: &DataFrame) -> Vec<DecadeData> {
    if let (Ok(disc_year_col), Ok(st_teff_col), Ok(hostname_col)) = (
        df.column("disc_year"),
        df.column("st_teff"),
        df.column("hostname"),
    ) {
        if let (
            Some(disc_year_series),
            Some(st_teff_series),
            Some(_hostname_series),
        ) = (
            disc_year_col.as_series(),
            st_teff_col.as_series(),
            hostname_col.as_series(),
        ) {
            if let Ok(disc_year_data) = disc_year_series.f64() {
                if let Ok(st_teff_data) = st_teff_series.f64() {
                    let mut decade_map: HashMap<i32, (u32, Vec<f64>)> =
                        HashMap::new();

                    // Group by decade and collect temperatures
                    for (i, opt_year) in disc_year_data.into_iter().enumerate() {
                        if let Some(year) = opt_year {
                            let decade = (year as i32 / 10) * 10;

                            if i < st_teff_data.len() {
                                let opt_temp = st_teff_data.get(i);
                                if let Some(temp) = opt_temp {
                                    if temp > 0.0 {
                                        let entry = decade_map
                                            .entry(decade)
                                            .or_insert((0, Vec::new()));
                                        entry.0 += 1;
                                        entry.1.push(temp);
                                    }
                                }
                            }
                        }
                    }

                    // Create result
                    let mut result = Vec::new();
                    for (decade, (count, temps)) in decade_map {
                        // Calculate median temperature
                        let median_temp = if !temps.is_empty() {
                            let mut sorted_temps = temps;
                            sorted_temps
                                .sort_by(|a, b| a.partial_cmp(b).unwrap());
                            let len = sorted_temps.len();
                            if len % 2 == 0 {
                                Some(
                                    (sorted_temps[len / 2 - 1]
                                        + sorted_temps[len / 2])
                                        / 2.0,
                                )
                            } else {
                                Some(sorted_temps[len / 2])
                            }
                        } else {
                            None
                        };

                        result.push(DecadeData {
                            decade,
                            stars_discovered: count,
                            discovery_methods: HashMap::new(), // Simplified for now
                            median_temp,
                        });
                    }

                    // Sort by decade
                    result.sort_by_key(|d| d.decade);
                    return result;
                }
            }
        }
    }

    vec![]
}

/// Compute catalog cross-match statistics
/// Analyzes coverage across different star catalogs (HD, HIP, TIC, GAIA)
pub fn catalog_crossmatch(df: &DataFrame) -> CatalogStats {
    let total_stars = df.height() as u32;

    // Count non-null values for each catalog
    let count_column = |col_name: &str| -> u32 {
        if let Ok(col) = df.column(col_name) {
            if let Some(series) = col.as_series() {
                return (series.len() - series.null_count()) as u32;
            }
        }
        0
    };

    let stars_with_hd = count_column("hd_name");
    let stars_with_hip = count_column("hip_name");
    let stars_with_tic = count_column("tic_id");
    let stars_with_gaia_dr2 = count_column("gaia_dr2_id");
    let stars_with_gaia_dr3 = count_column("gaia_dr3_id");

    CatalogStats {
        total_stars,
        hd_match_rate: (stars_with_hd as f64 / total_stars as f64) * 100.0,
        hip_match_rate: (stars_with_hip as f64 / total_stars as f64) * 100.0,
        tic_match_rate: (stars_with_tic as f64 / total_stars as f64) * 100.0,
        gaia_dr2_match_rate: (stars_with_gaia_dr2 as f64 / total_stars as f64)
            * 100.0,
        gaia_dr3_match_rate: (stars_with_gaia_dr3 as f64 / total_stars as f64)
            * 100.0,
        cross_match_matrix: Vec::new(), // Simplified for now
    }
}

/// Compute photometric statistics across multiple bands
/// Analyzes magnitude distributions in different photometric systems
pub fn photometric_statistics(df: &DataFrame) -> PhotometricStats {
    let photometric_bands = vec![
        ("sy_vmag", "V"),
        ("sy_bmag", "B"),
        ("sy_jmag", "J"),
        ("sy_hmag", "H"),
        ("sy_kmag", "K"),
        ("sy_gmag", "G"),
        ("sy_gaiamag", "Gaia"),
        ("sy_kepmag", "Kepler"),
    ];

    let mut band_stats = HashMap::new();

    for (col_name, band_name) in photometric_bands {
        if let Some(stats) = compute_band_stats(df, col_name) {
            band_stats.insert(band_name.to_string(), stats);
        }
    }

    // Compute color indices (simplified)
    let mut color_indices = HashMap::new();
    if band_stats.contains_key("B") && band_stats.contains_key("V") {
        color_indices.insert("B-V".to_string(), 0.0); // Simplified
    }
    if band_stats.contains_key("V") && band_stats.contains_key("K") {
        color_indices.insert("V-K".to_string(), 0.0); // Simplified
    }

    PhotometricStats {
        band_stats,
        color_indices,
    }
}

/// Compute statistics for a specific photometric band
fn compute_band_stats(df: &DataFrame, column: &str) -> Option<BandStats> {
    if let Ok(col) = df.column(column) {
        if let Some(series) = col.as_series() {
            if let Ok(f64_series) = series.f64() {
                let count = f64_series.len() as u32;
                if count > 0 {
                    let mean_mag = f64_series.mean().unwrap_or(0.0);
                    let median_mag = f64_series.median().unwrap_or(0.0);
                    let std_mag = f64_series.std(0).unwrap_or(0.0);
                    let min_mag = f64_series.min().unwrap_or(0.0);
                    let max_mag = f64_series.max().unwrap_or(0.0);

                    return Some(BandStats {
                        band_name: column.to_string(),
                        count,
                        mean_mag,
                        median_mag,
                        std_mag,
                        min_mag,
                        max_mag,
                    });
                }
            }
        }
    }

    None
}

// Simple aggregation functions for overview page statistics

/// Get total counts of stellar hosts and exoplanets
pub fn get_total_counts(
    stellarhosts_df: &DataFrame,
    exoplanets_df: &DataFrame,
) -> (usize, usize) {
    (stellarhosts_df.height(), exoplanets_df.height())
}

/// Calculate average stellar effective temperature
pub fn get_avg_temperature(df: &DataFrame) -> Option<f64> {
    df.column("st_teff")
        .ok()
        .and_then(|col| col.f64().ok())
        .and_then(|series| series.mean())
}

/// Calculate average distance to stellar systems
pub fn get_avg_distance(df: &DataFrame) -> Option<f64> {
    df.column("sy_dist")
        .ok()
        .and_then(|col| col.f64().ok())
        .and_then(|series| series.mean())
}

/// Get top N discovery methods with counts
pub fn get_discovery_methods(
    df: &DataFrame,
    limit: usize,
) -> Vec<(String, usize)> {
    let mut methods = HashMap::new();

    if let Ok(col) = df.column("discoverymethod") {
        if let Ok(str_col) = col.str() {
            for i in 0..str_col.len() {
                if let Some(method) = str_col.get(i) {
                    *methods.entry(method.to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    let mut methods_vec: Vec<_> = methods.into_iter().collect();
    methods_vec.sort_by(|a, b| b.1.cmp(&a.1));
    methods_vec.truncate(limit);
    methods_vec
}

/// Categorize planets by radius into size categories
pub fn get_planet_size_categories(df: &DataFrame) -> Vec<(String, usize)> {
    let mut categories = HashMap::new();

    if let Ok(col) = df.column("pl_rade") {
        if let Ok(f64_col) = col.f64() {
            for i in 0..f64_col.len() {
                if let Some(radius) = f64_col.get(i) {
                    let category = if radius < 1.0 {
                        "Sub-Earth (< 1 R⊕)"
                    } else if radius < 1.5 {
                        "Earth-like (1-1.5 R⊕)"
                    } else if radius < 2.5 {
                        "Super-Earth (1.5-2.5 R⊕)"
                    } else if radius < 4.0 {
                        "Neptune-like (2.5-4 R⊕)"
                    } else {
                        "Jupiter-like (> 4 R⊕)"
                    };
                    *categories.entry(category.to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    let mut categories_vec: Vec<_> = categories.into_iter().collect();
    // Sort by count descending
    categories_vec.sort_by(|a, b| b.1.cmp(&a.1));
    categories_vec
}
