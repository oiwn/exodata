use std::fs;
use std::time::Instant;

use anyhow::{Context, Error};
use clap::Parser;
use polars::prelude::*;

#[derive(Parser, Debug)]
#[command(about = "Measure Parquet load performance")]
struct Args {
    /// Parquet files to benchmark (comma-separated)
    #[arg(
        long,
        value_delimiter = ',',
        default_values_t = [
            String::from("data/stellarhosts.parquet"),
            String::from("data/exoplanets.parquet")
        ]
    )]
    parquet: Vec<String>,

    /// Optional row limit for faster benchmarking
    #[arg(long)]
    limit: Option<usize>,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    println!(
        "Parquet load benchmark (comma-separate paths to test multiple files):"
    );
    if let Some(limit) = args.limit {
        println!("  Row limit: {}", limit);
    }
    println!();

    for parquet in args.parquet {
        let parq_size = fs::metadata(&parquet).map(|m| m.len()).unwrap_or(0);
        let parquet_path = std::path::PathBuf::from(&parquet);
        let limit = args.limit;

        let (parq_df, parq_elapsed) = timed(|| {
            let file = std::fs::File::open(&parquet_path)?;
            let mut df = ParquetReader::new(file).finish()?;
            if let Some(n) = limit {
                df = df.head(Some(n));
            }
            Ok::<_, Error>(df)
        })
        .with_context(|| format!("Loading Parquet {}", parquet))?;

        println!(
            "  Parquet : {} ({:.1} MB)",
            &parquet,
            bytes_to_mb(parq_size)
        );
        println!(
            "    rows: {:>6}, cols: {:>4} | load: {:>6.2?}",
            parq_df.height(),
            parq_df.width(),
            parq_elapsed
        );
    }

    println!("Notes:");
    println!("  - Uses lazy scan + collect (column pruning + predicate pushdown if applied).");
    println!("  - Pass --limit to simulate sampling reads.");
    println!("  - Point --parquet to the converted files (e.g., data/stellarhosts.parquet,data/exoplanets.parquet).");

    Ok(())
}

fn timed<T, F>(f: F) -> Result<(T, std::time::Duration), Error>
where
    F: FnOnce() -> Result<T, Error>,
{
    let start = Instant::now();
    let result = f()?;
    Ok((result, start.elapsed()))
}

fn bytes_to_mb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}
