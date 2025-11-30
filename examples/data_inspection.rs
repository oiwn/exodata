// Data inspection example to understand the exact structure of the stellarhosts dataset.
// This will help fix the discovery timeline and other aggregation issues.
//
// Usage:
// ```bash
// cargo run --example data_inspection
// ```

use anyhow::Result;
use exoplanets_catalog::tables::stellarhosts::load_data_with_limit;
use polars::prelude::ChunkAgg;
use std::time::Instant;

fn main() -> Result<()> {
    println!("=== Stellar Hosts Data Inspection ===");
    println!();

    // Load the data
    println!("Loading stellar hosts data...");
    let start = Instant::now();
    let df = load_data_with_limit("data/stellarhosts.vot", Some(1000))?; // Load smaller sample for inspection
    let load_time = start.elapsed();

    println!(
        "✓ Loaded {} rows, {} columns in {:?}",
        df.height(),
        df.width(),
        load_time
    );
    println!();

    // List all columns to find discovery-related columns
    println!("=== ALL COLUMN NAMES ===");
    let columns = df.get_column_names();
    for (i, name) in columns.iter().enumerate() {
        println!("{:3}. {}", i + 1, name);
    }

    // Look for discovery-related columns
    println!("\n=== DISCOVERY-RELATED COLUMNS ===");
    let discovery_columns: Vec<String> = columns
        .iter()
        .filter(|&&name| {
            name.contains("disc")
                || name.contains("year")
                || name.contains("method")
        })
        .map(|name| name.to_string())
        .collect();

    if discovery_columns.is_empty() {
        println!("No obvious discovery columns found");
        println!("Checking for any columns that might contain discovery info...");

        // Check sample values of string columns for discovery info
        for name in columns.iter() {
            if let Ok(col) = df.column(name) {
                if let Some(series) = col.as_series() {
                    if let Ok(str_series) = series.str() {
                        // Check first few values for discovery-related terms
                        for i in 0..str_series.len().min(5) {
                            if let Some(val) = str_series.get(i) {
                                if val.to_lowercase().contains("transit")
                                    || val.to_lowercase().contains("radial")
                                    || val.to_lowercase().contains("timing")
                                    || val.to_lowercase().contains("imaging")
                                {
                                    println!("  Found possible discovery info in column '{}': {}", name, val);
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        for name in &discovery_columns {
            println!("  {}", name);
            if let Ok(col) = df.column(&name) {
                print_sample_values(&name, col);
            }
        }
    }

    // Check all date/time related columns
    println!("\n=== DATE/TIME RELATED COLUMNS ===");
    let date_columns: Vec<String> = columns
        .iter()
        .filter(|&&name| name.contains("date") || name.contains("time"))
        .map(|name| name.to_string())
        .collect();

    if date_columns.is_empty() {
        println!("No date/time columns found");
    } else {
        for name in &date_columns {
            println!("  {}", name);
            if let Ok(col) = df.column(&name) {
                print_sample_values(&name, col);
            }
        }
    }

    // Inspect key stellar property columns
    println!("\n=== KEY STELLAR PROPERTIES ===");
    let stellar_props = [
        "st_teff", "st_mass", "st_rad", "st_lum", "st_logg", "st_met", "st_age",
    ];

    for &name in &stellar_props {
        if let Ok(col) = df.column(name) {
            println!("  {}", name);
            print_sample_values(name, col);

            // Show statistics
            if let Some(series) = col.as_series() {
                if let Ok(f64_series) = series.f64() {
                    let total = series.len() - series.null_count();
                    let mean = f64_series.mean().unwrap_or(0.0);
                    let min = f64_series
                        .into_iter()
                        .filter_map(|x| x)
                        .fold(f64::INFINITY, |a, b| a.min(b));
                    let max = f64_series
                        .into_iter()
                        .filter_map(|x| x)
                        .fold(f64::NEG_INFINITY, |a, b| a.max(b));

                    println!("    → Stats: {} non-null values, mean={:.2}, range=[{:.2}, {:.2}]", 
                        total, mean, min, max);
                }
            }
        } else {
            println!("  {} (NOT FOUND)", name);
        }
    }

    // Check for planet count
    println!("\n=== PLANET COUNT COLUMNS ===");
    let planet_count_columns: Vec<String> = columns
        .iter()
        .filter(|&&name| name.contains("planet") || name.contains("pl_"))
        .map(|name| name.to_string())
        .collect();

    for name in &planet_count_columns {
        println!("  {}", name);
        if let Ok(col) = df.column(&name) {
            print_sample_values(&name, col);
        }
    }

    println!("\n=== SUMMARY ===");
    println!("Total columns: {}", columns.len());
    println!(
        "Missing discovery columns: {}",
        !discovery_columns.is_empty()
    );

    // Look for the most likely discovery-related column names
    let likely_discovery = columns
        .iter()
        .find(|&&name| name.contains("discovery") || name.contains("pl_disc"))
        .map(|name| name.to_string());

    if let Some(col_name) = likely_discovery {
        println!("Most likely discovery column: {}", col_name);
    } else {
        println!("No obvious discovery column found");
    }

    // Try to find year columns
    let year_columns: Vec<String> = columns
        .iter()
        .filter(|&&name| name.contains("year") || name.contains("yr"))
        .map(|name| name.to_string())
        .collect();

    if !year_columns.is_empty() {
        println!("Possible year columns: {:?}", year_columns);
    } else {
        println!("No obvious year columns found");
    }

    println!();
    println!("=== NEXT STEPS ===");
    println!("1. Discovery timeline fix: Look for actual discovery column names");
    println!("2. Consider using exoplanets dataset for discovery info");
    println!("3. Data preprocessing to fill missing stellar properties");

    Ok(())
}

fn print_sample_values(_name: &str, col: &polars::prelude::Column) {
    if let Some(series) = col.as_series() {
        if series.dtype().is_numeric() {
            if let Ok(f64_series) = series.f64() {
                let values: Vec<String> = f64_series
                    .into_iter()
                    .take(5)
                    .map(|x| {
                        x.map(|v| format!("{:.3}", v))
                            .unwrap_or_else(|| "NULL".to_string())
                    })
                    .collect();
                println!("    → Sample values: {}", values.join(", "));
            }
        } else if let Ok(str_series) = series.str() {
            let values: Vec<&str> = str_series
                .into_iter()
                .take(5)
                .map(|x| x.unwrap_or("NULL"))
                .collect();
            println!("    → Sample values: {}", values.join(", "));
        }
    } else {
        println!("    → Not a series");
    }
}
