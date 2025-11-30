// Simple non-GUI example demonstrating stellar hosts aggregations.
// This example loads the stellar hosts data and runs various aggregations to provide
// insights into the dataset, displaying results in a clean text format.
//
// Usage:
// ```bash
// cargo run --example simple_aggregation
// ```

use anyhow::Result;
use exoplanets_catalog::tables::aggregation::*;
use exoplanets_catalog::tables::stellarhosts::load_data_with_limit;
use polars::prelude::{
    col, lit, ChunkAgg, ChunkVar, IntoLazy, SortMultipleOptions,
};
use std::time::Instant;

fn main() -> Result<()> {
    println!("=== Stellar Hosts Aggregation Analysis ===");
    println!();

    // Load the data
    println!("Loading stellar hosts data...");
    let start_time = Instant::now();
    let df = load_data_with_limit("data/stellarhosts.vot", None)?;
    let load_time = start_time.elapsed();

    println!(
        "✓ Loaded {} rows, {} columns in {:?}",
        df.height(),
        df.width(),
        load_time
    );
    println!();

    // Run temperature distribution analysis
    println!("=== TEMPERATURE DISTRIBUTION ===");
    let start_time = Instant::now();
    let temp_data = temperature_distribution(&df)?;
    let temp_time = start_time.elapsed();

    println!("Temperature Range       | Stars | Percentage");
    println!("------------------------|-------|----------");
    for bin in &temp_data {
        println!(
            "{:<24} | {:>5} | {:>8.1}%",
            bin.range, bin.star_count, bin.percentage
        );
    }

    let total_stars: u32 = temp_data.iter().map(|b| b.star_count).sum();
    let mean_temp = temp_data
        .iter()
        .map(|b| (b.min_temp + b.max_temp) / 2.0 * b.star_count as f64)
        .sum::<f64>()
        / total_stars as f64;

    println!(
        "\nTotal Stars: {} | Mean Temperature: {:.0}K | Computed in {:?}",
        total_stars, mean_temp, temp_time
    );
    println!();

    // Run discovery timeline analysis
    println!("=== DISCOVERY TIMELINE ===");
    let start_time = Instant::now();
    let discovery_data = discovery_timeline(&df)?;
    let discovery_time = start_time.elapsed();

    println!("Decade   | Stars | Median Temp");
    println!("----------|-------|-------------");
    for decade in &discovery_data {
        let median_temp_str = decade
            .median_temp
            .map(|t| format!("{:.0}K", t))
            .unwrap_or_else(|| "N/A".to_string());
        println!(
            "{:>5}s   | {:>5} | {}",
            decade.decade, decade.stars_discovered, median_temp_str
        );
    }

    let total_discovered: u32 =
        discovery_data.iter().map(|d| d.stars_discovered).sum();
    println!(
        "\nTotal Stars Discovered: {} | Computed in {:?}",
        total_discovered, discovery_time
    );

    // Additional analysis: Discovery methods
    println!("\n=== DISCOVERY METHODS ===");
    if let Ok(discovery_methods_df) = df
        .clone()
        .lazy()
        .group_by(["discoverymethod"])
        .agg([lit(1).count().alias("count")])
        .filter(col("discoverymethod").is_not_null())
        .sort(
            vec!["count"],
            SortMultipleOptions::new().with_order_descending(true),
        )
        .collect()
    {
        println!("Method               | Stars | Percentage");
        println!("---------------------|-------|----------");
        let count_col = discovery_methods_df.column("count")?;
        let total = count_col.u32()?.into_iter().filter_map(|x| x).sum::<u32>();

        for row in 0..discovery_methods_df.height() {
            let method = discovery_methods_df
                .column("discoverymethod")?
                .str()?
                .get(row)
                .unwrap_or("N/A");
            let count = count_col.u32()?.get(row).unwrap_or(0);
            let percentage = (count as f64 / total as f64) * 100.0;
            println!("{:<20} | {:>5} | {:>8.1}%", method, count, percentage);
        }

        println!(
            "\nTotal Discovery Methods: {}",
            discovery_methods_df.height()
        );
    }

    // Additional analysis: Stellar properties summary
    println!("\n=== STELLAR PROPERTIES SUMMARY ===");
    let properties = vec![
        ("st_mass", "Stellar Mass", "M☉"),
        ("st_rad", "Stellar Radius", "R☉"),
        ("st_age", "Stellar Age", "Gyr"),
        ("st_logg", "Surface Gravity", "log(g)"),
        ("st_met", "Metallicity", "[Fe/H]"),
    ];

    println!("Property        | Count | Mean   | Std Dev | Range");
    println!(
        "----------------|-------|--------|---------|----------------------"
    );

    for (col_name, display_name, unit) in properties {
        if let Ok(col) = df.column(col_name) {
            if let Some(series) = col.as_series() {
                if let Ok(f64_series) = series.f64() {
                    let count = (series.len() - series.null_count()) as u32;
                    if count > 0 {
                        let mean = f64_series.mean().unwrap_or(0.0);
                        let std = f64_series.std(0).unwrap_or(0.0);
                        let min = f64_series
                            .into_iter()
                            .filter_map(|x| x)
                            .fold(f64::INFINITY, |a, b| a.min(b));
                        let max = f64_series
                            .into_iter()
                            .filter_map(|x| x)
                            .fold(f64::NEG_INFINITY, |a, b| a.max(b));

                        println!("{:<16} | {:>5} | {:>6.2} | {:>7.2} | [{:>6.2}, {:>6.2}] {}", 
                            display_name, 
                            count, 
                            mean, 
                            std, 
                            min, 
                            max, 
                            unit);
                    }
                }
            }
        }
    }

    println!();

    // Run catalog cross-matching analysis
    println!("=== CATALOG CROSS-MATCH ===");
    let start_time = Instant::now();
    let catalog_data = catalog_crossmatch(&df)?;
    let catalog_time = start_time.elapsed();

    println!("Catalog    | Coverage | Stars");
    println!("-----------|----------|-------");
    println!(
        "HD         |   {:>5.1}% | {}",
        catalog_data.hd_match_rate,
        (catalog_data.hd_match_rate * catalog_data.total_stars as f64 / 100.0)
            as u32
    );
    println!(
        "HIP        |   {:>5.1}% | {}",
        catalog_data.hip_match_rate,
        (catalog_data.hip_match_rate * catalog_data.total_stars as f64 / 100.0)
            as u32
    );
    println!(
        "TIC        |   {:>5.1}% | {}",
        catalog_data.tic_match_rate,
        (catalog_data.tic_match_rate * catalog_data.total_stars as f64 / 100.0)
            as u32
    );
    println!(
        "GAIA DR2   |   {:>5.1}% | {}",
        catalog_data.gaia_dr2_match_rate,
        (catalog_data.gaia_dr2_match_rate * catalog_data.total_stars as f64
            / 100.0) as u32
    );
    println!(
        "GAIA DR3   |   {:>5.1}% | {}",
        catalog_data.gaia_dr3_match_rate,
        (catalog_data.gaia_dr3_match_rate * catalog_data.total_stars as f64
            / 100.0) as u32
    );

    println!(
        "\nTotal Stars: {} | Computed in {:?}",
        catalog_data.total_stars, catalog_time
    );
    println!();

    // Run photometric statistics analysis
    println!("=== PHOTOMETRIC STATISTICS ===");
    let start_time = Instant::now();
    let photo_data = photometric_statistics(&df)?;
    let photo_time = start_time.elapsed();

    println!("Band     | Stars | Mean   | Median | Range");
    println!("---------|-------|--------|--------|----------------");
    for (band, stats) in &photo_data.band_stats {
        println!(
            "{:<9} | {:>5} | {:>6.2} | {:>6.2} | [{:>6.2}, {:>6.2}]",
            band,
            stats.count,
            stats.mean_mag,
            stats.median_mag,
            stats.min_mag,
            stats.max_mag
        );
    }

    println!(
        "\nTotal Bands: {} | Computed in {:?}",
        photo_data.band_stats.len(),
        photo_time
    );
    println!();

    // Memory usage summary
    println!("=== MEMORY USAGE ===");
    let memory_mb = df.estimated_size() as f64 / (1024.0 * 1024.0);
    println!("Dataset Memory: {:.2} MB", memory_mb);
    println!(
        "Memory per Star: {:.2} KB",
        memory_mb * 1024.0 / df.height() as f64
    );

    // Performance summary
    println!("\n=== PERFORMANCE SUMMARY ===");
    println!("Temperature Distribution: {:?}", temp_time);
    println!("Discovery Timeline: {:?}", discovery_time);
    println!("Catalog Cross-Match: {:?}", catalog_time);
    println!("Photometric Statistics: {:?}", photo_time);

    let total_computation =
        temp_time + discovery_time + catalog_time + photo_time;
    println!("Total Computation Time: {:?}", total_computation);

    Ok(())
}
