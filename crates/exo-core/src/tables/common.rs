use std::fs::File;

use anyhow::Error;
use polars::prelude::*;

/// Load a parquet file, optionally with a row limit for lightweight scans.
pub fn load_parquet(
    path: &str,
    limit: Option<usize>,
) -> Result<DataFrame, Error> {
    let file = File::open(path)?;
    let mut df = ParquetReader::new(file).finish()?;
    if let Some(n) = limit {
        df = df.head(Some(n));
    }
    Ok(df)
}

/// Load data from a parquet file (convenience wrapper)
pub fn load_data(path: &str) -> Result<DataFrame, Error> {
    load_parquet(path, None)
}

/// Load data from a parquet file with a row limit
pub fn load_data_with_limit(
    path: &str,
    limit: Option<usize>,
) -> Result<DataFrame, Error> {
    load_parquet(path, limit)
}

/// Common functions for table operations
pub fn count_non_null_values(
    df: &DataFrame,
    col_name: &str,
) -> Result<usize, Error> {
    if let Ok(col) = df.column(col_name) {
        if let Some(series) = col.as_series() {
            if let Ok(f64_series) = series.f64() {
                Ok(f64_series.len())
            } else {
                // For non-float columns, count non-null values
                Ok(series.len() - series.null_count())
            }
        } else {
            Ok(0)
        }
    } else {
        Ok(0)
    }
}

/// Get basic statistics for a numeric column
pub fn get_numeric_stats(
    df: &DataFrame,
    col_name: &str,
) -> Result<Option<NumericStats>, Error> {
    if let Ok(col) = df.column(col_name) {
        if let Some(series) = col.as_series() {
            if let Ok(f64_series) = series.f64() {
                if f64_series.len() > 0 {
                    return Ok(Some(NumericStats {
                        count: f64_series.len(),
                        mean: f64_series.mean().unwrap_or(0.0),
                        median: f64_series.median().unwrap_or(0.0),
                        std: f64_series.std(0).unwrap_or(0.0),
                        min: f64_series.min().unwrap_or(0.0),
                        max: f64_series.max().unwrap_or(0.0),
                    }));
                }
            }
        }
    }
    Ok(None)
}

/// Statistics for numeric columns
pub struct NumericStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std: f64,
    pub min: f64,
    pub max: f64,
}

/// Create a simple histogram for a numeric column
pub fn create_histogram(
    df: &DataFrame,
    col_name: &str,
    min_val: f64,
    max_val: f64,
    bins: usize,
) -> Result<Vec<HistogramBin>, Error> {
    if let Ok(col) = df.column(col_name) {
        if let Some(series) = col.as_series() {
            if let Ok(f64_series) = series.f64() {
                let bin_width = (max_val - min_val) / bins as f64;
                let mut bin_counts = vec![0; bins];
                let mut bin_edges = Vec::new();

                // Create bin edges
                for i in 0..=bins {
                    bin_edges.push(min_val + i as f64 * bin_width);
                }

                // Count values in each bin
                for opt_val in f64_series.into_iter() {
                    if let Some(val) = opt_val {
                        if val >= min_val && val <= max_val {
                            let bin_index =
                                ((val - min_val) / bin_width) as usize;
                            if bin_index < bins {
                                bin_counts[bin_index] += 1;
                            }
                        }
                    }
                }

                // Create histogram bins
                let mut histogram = Vec::new();
                for i in 0..bins {
                    histogram.push(HistogramBin {
                        min: bin_edges[i],
                        max: bin_edges[i + 1],
                        count: bin_counts[i],
                    });
                }

                return Ok(histogram);
            }
        }
    }

    Ok(vec![])
}

/// A single bin in a histogram
pub struct HistogramBin {
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

/// Print a histogram to the console
pub fn print_histogram(histogram: &[HistogramBin], max_bar_width: usize) {
    let max_count = histogram.iter().map(|bin| bin.count).max().unwrap_or(1);

    for bin in histogram {
        let bar_width = if max_count > 0 {
            (bin.count * max_bar_width) / max_count
        } else {
            0
        };
        let bar = "█".repeat(bar_width);

        println!(
            "{:8.1} - {:8.1} | {:5} | {}",
            bin.min, bin.max, bin.count, bar
        );
    }
}
