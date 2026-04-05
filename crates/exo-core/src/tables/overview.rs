// Overview aggregation functions for stellar hosts and exoplanets data analysis.
// These functions are used by the overview page and other parts of the application.

use polars::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Default)]
struct PlanetAggregate {
    discovery_methods: HashMap<String, usize>,
    radii: Vec<f64>,
    orbital_periods: Vec<f64>,
    discovery_years: Vec<i32>,
}

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
    if let Ok(st_teff_col) = df.column("st_teff")
        && let Some(st_teff_series) = st_teff_col.as_series()
        && let Ok(st_teff_data) = st_teff_series.f64()
    {
        let total_stars = st_teff_data.len() as f64;
        let mut bin_counts = [0; 7]; // 7 bins from 3000-10000K

        // Count values in each bin
        for temp in st_teff_data.into_iter().flatten() {
            if (3000.0..=10000.0).contains(&temp) {
                let bin_index = ((temp - 3000.0) / 1000.0) as usize;
                if bin_index < 7 {
                    bin_counts[bin_index] += 1;
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

    vec![]
}

/// Compute discovery timeline by decade
/// Analyzes when stars were discovered and their properties
pub fn discovery_timeline(df: &DataFrame) -> Vec<DecadeData> {
    if let (Ok(disc_year_col), Ok(st_teff_col), Ok(hostname_col)) = (
        df.column("disc_year"),
        df.column("st_teff"),
        df.column("hostname"),
    ) && let (Some(disc_year_series), Some(st_teff_series)) =
        (disc_year_col.as_series(), st_teff_col.as_series())
        && hostname_col.as_series().is_some()
        && let Ok(disc_year_data) = disc_year_series.f64()
        && let Ok(st_teff_data) = st_teff_series.f64()
    {
        let mut decade_map: HashMap<i32, (u32, Vec<f64>)> = HashMap::new();

        // Group by decade and collect temperatures
        for (i, opt_year) in disc_year_data.into_iter().enumerate() {
            if let Some(year) = opt_year {
                let decade = (year as i32 / 10) * 10;

                if i < st_teff_data.len()
                    && let Some(temp) = st_teff_data.get(i)
                    && temp > 0.0
                {
                    let entry =
                        decade_map.entry(decade).or_insert((0, Vec::new()));
                    entry.0 += 1;
                    entry.1.push(temp);
                }
            }
        }

        // Create result
        let mut result = Vec::new();
        for (decade, (count, temps)) in decade_map {
            // Calculate median temperature
            let median_temp = if !temps.is_empty() {
                let mut sorted_temps = temps;
                sorted_temps.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let len = sorted_temps.len();
                if len % 2 == 0 {
                    Some(
                        (sorted_temps[len / 2 - 1] + sorted_temps[len / 2]) / 2.0,
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

    vec![]
}

/// Compute catalog cross-match statistics
/// Analyzes coverage across different star catalogs (HD, HIP, TIC, GAIA)
pub fn catalog_crossmatch(df: &DataFrame) -> CatalogStats {
    let total_stars = df.height() as u32;

    // Count non-null values for each catalog
    let count_column = |col_name: &str| -> u32 {
        if let Ok(col) = df.column(col_name)
            && let Some(series) = col.as_series()
        {
            return (series.len() - series.null_count()) as u32;
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
    if let Ok(col) = df.column(column)
        && let Some(series) = col.as_series()
        && let Ok(f64_series) = series.f64()
    {
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

    None
}

// Simple aggregation functions for overview page statistics

/// Get total counts of stellar hosts and exoplanets
pub fn get_total_counts(
    stellarhosts_df: &DataFrame,
    exoplanets_df: &DataFrame,
) -> (usize, usize) {
    let stellarhosts_total = distinct_non_null_count(stellarhosts_df, "hostname");
    let exoplanets_total = distinct_non_null_count(exoplanets_df, "pl_name");

    (stellarhosts_total, exoplanets_total)
}

fn distinct_non_null_count(df: &DataFrame, column: &str) -> usize {
    df.column(column)
        .ok()
        .and_then(|col| col.as_series())
        .and_then(|series| series.drop_nulls().n_unique().ok())
        .unwrap_or_else(|| df.height())
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
    let planets = build_planet_aggregates(df);
    let mut methods = HashMap::new();

    for aggregate in planets.values() {
        if let Some(method) = canonical_string(&aggregate.discovery_methods) {
            *methods.entry(method).or_insert(0) += 1;
        }
    }

    let mut methods_vec: Vec<_> = methods.into_iter().collect();
    methods_vec.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    methods_vec.truncate(limit);
    methods_vec
}

/// Categorize planets by radius into size categories
pub fn get_planet_size_categories(df: &DataFrame) -> Vec<(String, usize)> {
    let planets = build_planet_aggregates(df);
    let mut categories = HashMap::new();

    for aggregate in planets.values() {
        if let Some(radius) = median_f64(&aggregate.radii) {
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

    let mut categories_vec: Vec<_> = categories.into_iter().collect();
    // Sort by count descending
    categories_vec.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    categories_vec
}

/// Get top N discovery years with distinct planet counts
pub fn get_discovery_year_counts(
    df: &DataFrame,
    limit: usize,
) -> Vec<(String, usize)> {
    let planets = build_planet_aggregates(df);
    let mut years = HashMap::new();

    for aggregate in planets.values() {
        if let Some(year) = aggregate.discovery_years.iter().min() {
            *years.entry(year.to_string()).or_insert(0) += 1;
        }
    }

    let mut years_vec: Vec<_> = years.into_iter().collect();
    years_vec.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0)));
    years_vec.truncate(limit);
    years_vec
}

/// Group distinct planets by canonical orbital period bucket
pub fn get_orbital_period_buckets(df: &DataFrame) -> Vec<(String, usize)> {
    let planets = build_planet_aggregates(df);
    let mut buckets = HashMap::new();

    for aggregate in planets.values() {
        if let Some(period) = median_f64(&aggregate.orbital_periods) {
            let bucket = if period < 1.0 {
                "< 1 day"
            } else if period < 10.0 {
                "1-10 days"
            } else if period < 100.0 {
                "10-100 days"
            } else if period < 1000.0 {
                "100-1000 days"
            } else {
                "> 1000 days"
            };
            *buckets.entry(bucket.to_string()).or_insert(0) += 1;
        }
    }

    let mut buckets_vec: Vec<_> = buckets.into_iter().collect();
    buckets_vec.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    buckets_vec
}

fn build_planet_aggregates(df: &DataFrame) -> HashMap<String, PlanetAggregate> {
    let Ok(pl_name_col) = df.column("pl_name") else {
        return HashMap::new();
    };
    let discovery_method_col = df.column("discoverymethod").ok();
    let radius_col = df.column("pl_rade").ok();
    let orbital_period_col = df.column("pl_orbper").ok();
    let discovery_year_col = df.column("disc_year").ok();

    let mut planets = HashMap::new();

    for row_idx in 0..df.height() {
        let Some(pl_name) = string_value_at(pl_name_col, row_idx) else {
            continue;
        };
        let aggregate = planets
            .entry(pl_name)
            .or_insert_with(PlanetAggregate::default);

        if let Some(col) = discovery_method_col.as_ref()
            && let Some(method) = string_value_at(col, row_idx)
        {
            *aggregate.discovery_methods.entry(method).or_insert(0) += 1;
        }

        if let Some(col) = radius_col.as_ref()
            && let Some(radius) = float_value_at(col, row_idx)
            && radius.is_finite()
        {
            aggregate.radii.push(radius);
        }

        if let Some(col) = orbital_period_col.as_ref()
            && let Some(period) = float_value_at(col, row_idx)
            && period.is_finite()
            && period >= 0.0
        {
            aggregate.orbital_periods.push(period);
        }

        if let Some(col) = discovery_year_col.as_ref()
            && let Some(year) = year_value_at(col, row_idx)
        {
            aggregate.discovery_years.push(year);
        }
    }

    planets
}

fn string_value_at(col: &Column, row_idx: usize) -> Option<String> {
    match col.get(row_idx).ok()? {
        AnyValue::String(value) => Some(value.to_string()),
        AnyValue::StringOwned(value) => Some(value.as_str().to_string()),
        _ => None,
    }
}

fn float_value_at(col: &Column, row_idx: usize) -> Option<f64> {
    match col.get(row_idx).ok()? {
        AnyValue::Float64(value) => Some(value),
        AnyValue::Float32(value) => Some(value as f64),
        AnyValue::Int64(value) => Some(value as f64),
        AnyValue::Int32(value) => Some(value as f64),
        AnyValue::UInt64(value) => Some(value as f64),
        AnyValue::UInt32(value) => Some(value as f64),
        _ => None,
    }
}

fn year_value_at(col: &Column, row_idx: usize) -> Option<i32> {
    match col.get(row_idx).ok()? {
        AnyValue::Int64(value) => i32::try_from(value).ok(),
        AnyValue::Int32(value) => Some(value),
        AnyValue::UInt64(value) => i32::try_from(value).ok(),
        AnyValue::UInt32(value) => i32::try_from(value).ok(),
        AnyValue::Float64(value) if value.is_finite() => Some(value as i32),
        AnyValue::Float32(value) if value.is_finite() => Some(value as i32),
        _ => None,
    }
}

fn canonical_string(values: &HashMap<String, usize>) -> Option<String> {
    let mut sorted: Vec<_> = values.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    sorted.first().map(|(value, _)| (*value).clone())
}

fn median_f64(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let len = sorted.len();

    if len % 2 == 0 {
        Some((sorted[len / 2 - 1] + sorted[len / 2]) / 2.0)
    } else {
        Some(sorted[len / 2])
    }
}

#[cfg(test)]
mod tests {
    use super::{
        get_discovery_methods, get_discovery_year_counts,
        get_orbital_period_buckets, get_planet_size_categories, get_total_counts,
    };
    use polars::df;

    #[test]
    fn total_counts_use_distinct_host_and_planet_names() {
        let stellarhosts_df = df!(
            "hostname" => &[Some("HD 1"), Some("HD 1"), Some("HD 2"), None]
        )
        .unwrap();
        let exoplanets_df = df!(
            "pl_name" => &[Some("Planet A"), Some("Planet A"), Some("Planet B"), None]
        )
        .unwrap();

        let (stellarhosts_total, exoplanets_total) =
            get_total_counts(&stellarhosts_df, &exoplanets_df);

        assert_eq!(stellarhosts_total, 2);
        assert_eq!(exoplanets_total, 2);
    }

    #[test]
    fn discovery_methods_use_one_canonical_method_per_planet() {
        let exoplanets_df = df!(
            "pl_name" => &[
                "Planet A",
                "Planet A",
                "Planet A",
                "Planet B",
                "Planet B",
                "Planet C",
            ],
            "discoverymethod" => &[
                "Transit",
                "Transit",
                "Radial Velocity",
                "Radial Velocity",
                "Radial Velocity",
                "Imaging",
            ]
        )
        .unwrap();

        let methods = get_discovery_methods(&exoplanets_df, 10);

        assert_eq!(
            methods,
            vec![
                ("Imaging".to_string(), 1),
                ("Radial Velocity".to_string(), 1),
                ("Transit".to_string(), 1),
            ]
        );
    }

    #[test]
    fn planet_size_categories_use_one_canonical_radius_per_planet() {
        let exoplanets_df = df!(
            "pl_name" => &["Planet A", "Planet A", "Planet B", "Planet B", "Planet C"],
            "pl_rade" => &[1.2, 1.4, 3.1, 3.7, 5.2]
        )
        .unwrap();

        let categories = get_planet_size_categories(&exoplanets_df);

        assert_eq!(
            categories,
            vec![
                ("Earth-like (1-1.5 R⊕)".to_string(), 1),
                ("Jupiter-like (> 4 R⊕)".to_string(), 1),
                ("Neptune-like (2.5-4 R⊕)".to_string(), 1),
            ]
        );
    }

    #[test]
    fn discovery_years_use_earliest_year_per_planet() {
        let exoplanets_df = df!(
            "pl_name" => &["Planet A", "Planet A", "Planet B", "Planet C", "Planet C"],
            "disc_year" => &[2016i32, 2018i32, 2014i32, 2021i32, 2020i32]
        )
        .unwrap();

        let years = get_discovery_year_counts(&exoplanets_df, 10);

        assert_eq!(
            years,
            vec![
                ("2020".to_string(), 1),
                ("2016".to_string(), 1),
                ("2014".to_string(), 1),
            ]
        );
    }

    #[test]
    fn orbital_period_buckets_use_one_canonical_period_per_planet() {
        let exoplanets_df = df!(
            "pl_name" => &[
                "Planet A",
                "Planet A",
                "Planet B",
                "Planet C",
                "Planet D",
                "Planet E",
            ],
            "pl_orbper" => &[0.8, 0.9, 5.0, 55.0, 500.0, 5000.0]
        )
        .unwrap();

        let buckets = get_orbital_period_buckets(&exoplanets_df);

        assert_eq!(
            buckets,
            vec![
                ("1-10 days".to_string(), 1),
                ("10-100 days".to_string(), 1),
                ("100-1000 days".to_string(), 1),
                ("< 1 day".to_string(), 1),
                ("> 1000 days".to_string(), 1),
            ]
        );
    }
}
