use anyhow::Error;
use clap::Parser;
use polars::prelude::*;

use exoplanets_catalog::tables::common::load_parquet;

#[derive(Parser, Debug)]
#[command(about = "Inspect stellar hosts dataset (coverage, photometry, stellar properties)")]
struct Args {
    /// Parquet file to inspect
    #[arg(long, default_value = "data/stellarhosts.parquet")]
    file: String,

    /// Optional column name substring to search for (case-insensitive)
    #[arg(long)]
    search: Option<String>,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let df = load_parquet(&args.file, None)?;

    println!("Stellar Hosts Inspection");
    println!("File: {}", args.file);
    println!("Rows: {}, Cols: {}", df.height(), df.width());
    println!();

    if let Some(q) = args.search.as_ref() {
        search_columns(&df, q);
        println!();
    }

    print_identifier_coverage(&df);
    println!();

    print_photometry_stats(&df);
    println!();

    print_stellar_property_stats(&df);
    println!();

    print_planet_count_distribution(&df);

    Ok(())
}

fn search_columns(df: &DataFrame, query: &str) {
    let q = query.to_lowercase();
    let mut matches: Vec<&str> = df
        .get_column_names()
        .iter()
        .filter_map(|name| {
            if name.to_lowercase().contains(&q) {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect();
    matches.sort();

    println!("Columns matching \"{}\":", query);
    if matches.is_empty() {
        println!("  (none)");
    } else {
        for name in matches {
            println!("  {}", name);
        }
    }
}

fn print_identifier_coverage(df: &DataFrame) {
    let id_cols = [
        ("hostname", "Host name"),
        ("hd_name", "HD"),
        ("hip_name", "HIP"),
        ("tic_id", "TIC"),
        ("gaia_id", "GAIA"),
    ];

    println!("Identifier coverage (non-null rows):");
    for (col, label) in id_cols {
        if let Some((count, pct)) = coverage(df, col) {
            println!("  {:<8} {:>8} ({:>5.1}%)", label, count, pct);
        }
    }
}

fn print_photometry_stats(df: &DataFrame) {
    let phot_cols = [
        "sy_vmag", "sy_bmag", "sy_jmag", "sy_hmag", "sy_kmag", "sy_gmag",
        "sy_gaiamag", "sy_kepmag",
    ];

    println!("Photometric bands (non-null counts):");
    for col in phot_cols {
        if let Some((count, pct)) = coverage(df, col) {
            println!("  {:<10} {:>8} ({:>5.1}%)", col, count, pct);
        }
    }
}

fn print_stellar_property_stats(df: &DataFrame) {
    let cols = [
        ("st_teff", "Teff (K)"),
        ("st_mass", "Mass (M☉)"),
        ("st_rad", "Radius (R☉)"),
        ("st_logg", "logg"),
        ("st_lum", "Luminosity"),
        ("st_age", "Age (Gyr)"),
        ("st_met", "Metallicity"),
    ];

    println!("Stellar property statistics:");
    for (col, label) in cols {
        if let Some(stats) = numeric_stats(df, col) {
            println!(
                "  {:<12} count {:>6}, mean {:>10.3}, median {:>10.3}, std {:>10.3}, min {:>10.3}, max {:>10.3}",
                label,
                stats.count,
                stats.mean,
                stats.median,
                stats.std,
                stats.min,
                stats.max
            );
        }
    }
}

fn print_planet_count_distribution(df: &DataFrame) {
    println!("Host planet count distribution (sy_pnum):");
    match value_counts(df, "sy_pnum", 10) {
        Ok(table) => {
            for row in table.iter() {
                println!("  {:>5} planets : {:>6}", row.value, row.count);
            }
        }
        Err(e) => {
            println!("  unable to compute distribution: {}", e);
        }
    }
}

fn coverage(df: &DataFrame, col: &str) -> Option<(usize, f64)> {
    if let Ok(series) = df.column(col) {
        let total = series.len();
        if total == 0 {
            return None;
        }
        let non_null = total - series.null_count();
        let pct = (non_null as f64 / total as f64) * 100.0;
        Some((non_null, pct))
    } else {
        None
    }
}

struct NumericStats {
    count: usize,
    mean: f64,
    median: f64,
    std: f64,
    min: f64,
    max: f64,
}

fn numeric_stats(df: &DataFrame, col: &str) -> Option<NumericStats> {
    let series = df.column(col).ok()?;
    let cast = series.cast(&DataType::Float64).ok()?;
    let s = cast.f64().ok()?;
    if s.len() == 0 {
        return None;
    }
    Some(NumericStats {
        count: s.len(),
        mean: s.mean().unwrap_or(0.0),
        median: s.median().unwrap_or(0.0),
        std: s.std(0).unwrap_or(0.0),
        min: s.min().unwrap_or(0.0),
        max: s.max().unwrap_or(0.0),
    })
}

struct ValueCount {
    value: i64,
    count: u32,
}

fn value_counts(
    df: &DataFrame,
    column: &str,
    top_n: usize,
) -> Result<Vec<ValueCount>, Error> {
    let counts_df = df
        .clone()
        .group_by([column])?
        .count()?
        .sort(
            ["count"],
            SortMultipleOptions {
                descending: vec![true],
                ..Default::default()
            },
        )?
        .head(Some(top_n));

    let values_col = counts_df
        .column(column)?
        .cast(&DataType::Int64)?;
    let values = values_col.i64()?;
    let counts = counts_df.column("count")?.u32()?;

    Ok(values
        .into_iter()
        .zip(counts.into_iter())
        .filter_map(|(value, count)| match (value, count) {
            (Some(v), Some(c)) => Some(ValueCount {
                value: v,
                count: c,
            }),
            _ => None,
        })
        .collect())
}
