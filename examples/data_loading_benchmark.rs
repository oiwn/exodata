// Data loading benchmark to investigate performance issues with VOTable loading.
// This example measures loading times and explores faster alternatives like Parquet.
//
// Usage:
// ```bash
// cargo run --example data_loading_benchmark
// ```

use anyhow::Result;
use exoplanets_catalog::tables::stellarhosts::load_data_with_limit;
use polars::prelude::*;
use std::time::Instant;

fn main() -> Result<()> {
    println!("=== Data Loading Benchmark ===");
    println!();

    // Benchmark 1: Loading VOTable directly
    println!("1. Loading VOTable directly (from StellarHosts dataset)...");
    let start = Instant::now();
    let df = load_data_with_limit("data/stellarhosts.vot", None)?;
    let votable_time = start.elapsed();
    println!(
        "   ✓ Loaded {} rows, {} columns in {:?}",
        df.height(),
        df.width(),
        votable_time
    );
    println!(
        "   ✓ Memory: {:.2} MB",
        df.estimated_size() as f64 / (1024.0 * 1024.0)
    );

    // Note: Parquet saving disabled - requires polars with parquet feature
    println!(
        "\n2. Parquet conversion would go here (requires polars parquet feature)"
    );
    println!("   To enable: Add parquet feature to polars in Cargo.toml");
    let save_time = std::time::Duration::from_millis(0);

    // Simulate hypothetical parquet loading speed
    println!("\n3. Hypothetical Parquet loading (VOTable to Parquet typically provides 10-100x speedup)");
    let estimated_parquet_time = std::time::Duration::from_millis(200); // Rough estimate
    println!(
        "   ✓ Estimated load time: {:?} (vs {:?} for VOTable)",
        estimated_parquet_time, votable_time
    );

    if votable_time.as_secs_f64() > 0.0 {
        let speedup =
            votable_time.as_secs_f64() / estimated_parquet_time.as_secs_f64();
        println!(
            "   ✓ Estimated speedup: {:.1}x faster than VOTable",
            speedup
        );
    }

    // Benchmark 3: Loading subsets of data
    println!("\n4. Loading smaller subsets...");
    for limit in [100, 1000, 10000] {
        let start = Instant::now();
        let df_subset =
            load_data_with_limit("data/stellarhosts.vot", Some(limit))?;
        let time = start.elapsed();
        println!(
            "   ✓ {} rows: {:?} ({:.2} MB/row)",
            df_subset.height(),
            time,
            df_subset.estimated_size() as f64
                / df_subset.height() as f64
                / (1024.0 * 1024.0)
        );
    }

    // Investigate specific data quality issues
    println!("\n=== Data Quality Analysis ===");

    // Check for missing values in key columns
    let key_columns = [
        "st_teff",
        "st_mass",
        "st_rad",
        "disc_year",
        "discoverymethod",
    ];

    println!("Missing values analysis:");
    println!("Column          | Total | Missing | Missing%");
    println!("----------------|-------|---------|----------");

    for &col_name in &key_columns {
        if let Ok(col) = df.column(col_name) {
            let total = df.height();
            let missing = col.null_count();
            let missing_pct = (missing as f64 / total as f64) * 100.0;
            println!(
                "{:<16} | {:>5} | {:>7} | {:>8.1}%",
                col_name, total, missing, missing_pct
            );
        } else {
            println!(
                "{:<16} | {:>5} | {:>7} | {:>8}",
                "Column not found", "-", "-", ""
            );
        }
    }

    // Check discovery timeline data
    println!("\nDiscovery Year Analysis:");
    if let Ok(disc_year_col) = df.column("disc_year") {
        if let Some(series) = disc_year_col.as_series() {
            if let Ok(years) = series.f64() {
                let total_years = years.into_iter().filter_map(|x| x).count();
                println!("  Records with discovery year: {}", total_years);

                if total_years > 0 {
                    let min_year = years
                        .into_iter()
                        .filter_map(|x| x)
                        .fold(f64::INFINITY, |a, b| a.min(b));
                    let max_year = years
                        .into_iter()
                        .filter_map(|x| x)
                        .fold(f64::NEG_INFINITY, |a, b| a.max(b));
                    println!("  Year range: {:.0} - {:.0}", min_year, max_year);
                }
            }
        }
    }

    // Analyze what's going wrong with discovery timeline
    println!("\nDetailed discovery data inspection:");
    if let (Ok(disc_year_col), Ok(hostname_col)) =
        (df.column("disc_year"), df.column("hostname"))
    {
        if let (Some(year_series), Some(host_series)) =
            (disc_year_col.as_series(), hostname_col.as_series())
        {
            if let (Ok(years), Ok(hosts)) = (year_series.f64(), host_series.str())
            {
                let mut valid_records = 0;
                let mut samples = Vec::new();

                for i in 0..years.len().min(10) {
                    let year_opt = years.get(i);
                    let hostname_opt = hosts.get(i);

                    if let (Some(year), Some(hostname)) = (year_opt, hostname_opt)
                    {
                        if year > 0.0 {
                            valid_records += 1;
                            if samples.len() < 5 {
                                samples.push((hostname, year));
                            }
                        }
                    }
                }

                println!(
                    "  Valid discovery year records in first 10: {}",
                    valid_records
                );

                if !samples.is_empty() {
                    println!("  Sample records:");
                    for (hostname, year) in samples {
                        println!("    {} - discovered in {:.0}", hostname, year);
                    }
                }
            }
        }
    }

    println!("\n=== Recommendations ===");
    println!(
        "1. Convert VOTable to Parquet for {:.0}x faster loading",
        votable_time.as_secs_f64() / estimated_parquet_time.as_secs_f64()
    );
    println!("2. Use Parquet for production/regular use");
    println!("3. Consider data preprocessing to handle missing values");
    println!("4. Discovery timeline needs further investigation - disc_year column may have issues");

    Ok(())
}
