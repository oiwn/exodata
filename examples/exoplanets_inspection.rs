use anyhow::Error;
use clap::Parser;
use polars::prelude::*;
use polars::prelude::AnyValue;

use exoplanets_catalog::tables::common::load_parquet;

#[derive(Parser, Debug)]
#[command(about = "Inspect exoplanets dataset (discovery timeline/methods, orbital/physical stats)")]
struct Args {
    /// Parquet file to inspect
    #[arg(long, default_value = "data/exoplanets.parquet")]
    file: String,

    /// Optional column name substring to search for (case-insensitive)
    #[arg(long)]
    search: Option<String>,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let df = load_parquet(&args.file, None)?;

    println!("Exoplanets Inspection");
    println!("File: {}", args.file);
    println!("Rows: {}, Cols: {}", df.height(), df.width());
    println!();

    if let Some(q) = args.search.as_ref() {
        search_columns(&df, q);
        println!();
    }

    print_discovery_timeline(&df)?;
    println!();

    print_discovery_methods(&df)?;
    println!();

    print_orbital_stats(&df);
    println!();

    print_physical_stats(&df);

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

fn print_discovery_timeline(df: &DataFrame) -> Result<(), Error> {
    let timeline = df
        .clone()
        .group_by(["disc_year"])?
        .count()?
        .sort(
            ["disc_year"],
            SortMultipleOptions {
                descending: vec![false],
                ..Default::default()
            },
        )?;

    println!("Discovery timeline (disc_year):");
    let years = timeline.column("disc_year")?.i32()?;
    let counts = timeline.column("count")?.u32()?;
    for (year, count) in years.into_iter().zip(counts) {
        if let (Some(y), Some(c)) = (year, count) {
            println!("  {:>4}: {:>6}", y, c);
        }
    }
    Ok(())
}

fn print_discovery_methods(df: &DataFrame) -> Result<(), Error> {
    let methods = df
        .clone()
        .group_by(["discoverymethod"])?
        .count()?
        .sort(
            ["count"],
            SortMultipleOptions {
                descending: vec![true],
                ..Default::default()
            },
        )?
        .head(Some(15));

    println!("Discovery methods:");
    let names_col = methods.column("discoverymethod")?;
    let counts = methods.column("count")?.u32()?;
    let names_iter = names_col.to_owned().into_series().iter();
    for (name, count) in names_iter.zip(counts) {
        if let Some(c) = count {
            let label = match name {
                AnyValue::Null => "<unknown>".to_string(),
                _ => name.to_string(),
            };
            println!("  {:<25} {:>6}", label, c);
        }
    }
    Ok(())
}

fn print_orbital_stats(df: &DataFrame) {
    let cols = [
        ("pl_orbper", "Orbital period (days)"),
        ("pl_orbsmax", "Semi-major axis (AU)"),
        ("pl_orbeccen", "Eccentricity"),
        ("pl_eqt", "Equilibrium temp (K)"),
    ];

    println!("Orbital statistics:");
    for (col, label) in cols {
        if let Some(stats) = numeric_stats(df, col) {
            println!(
                "  {:<24} count {:>6}, mean {:>10.3}, median {:>10.3}, std {:>10.3}, min {:>10.3}, max {:>10.3}",
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

fn print_physical_stats(df: &DataFrame) {
    let cols = [
        ("pl_masse", "Mass (Earth)"),
        ("pl_massj", "Mass (Jupiter)"),
        ("pl_rade", "Radius (Earth)"),
        ("pl_radj", "Radius (Jupiter)"),
    ];

    println!("Physical statistics:");
    for (col, label) in cols {
        if let Some(stats) = numeric_stats(df, col) {
            println!(
                "  {:<18} count {:>6}, mean {:>10.3}, median {:>10.3}, std {:>10.3}, min {:>10.3}, max {:>10.3}",
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
